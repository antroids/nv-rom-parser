// SPDX-License-Identifier: MIT

use crate::nvidia::bit;
use crate::nvidia::dcb;
use binread::{BinRead, BinReaderExt, BinResult, ReadOptions};
use log::trace;
use serde::Serialize;
use std::any::type_name;
use std::fmt::{Debug, Display, Formatter};
use std::io::{Read, Seek, SeekFrom};

pub mod cursor;
pub mod firmware;
pub mod nvidia;
pub mod pci_efi;
pub mod pci_legacy;

const FIRMWARE_REGION_ALIGN: u64 = 512;
const FIRMWARE_REGION_STRUCTURE_ALIGN: u64 = 1;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: `{0}`")]
    IOError(#[from] std::io::Error),
    #[error("Firmware file has invalid format: `{0}`")]
    InvalidFormat(String),
    #[error("Binary format parsing Error: `{0}`")]
    BinReadError(#[from] binread::Error),
    #[error("Error: `{0}`")]
    ErrorMessage(String),
}

fn stream_position<R: Read + Seek>(reader: &mut R, _ro: &ReadOptions, _: ()) -> BinResult<u64> {
    Ok(reader.stream_position()?)
}

fn align(source: &mut impl Seek, alignment: u64) -> Result<()> {
    let offset = source.stream_position()?;
    let aligned_offset = offset + (alignment - 1) & !(alignment - 1);
    trace!(
        "Align: unaligned position {} aligned position {}",
        offset,
        aligned_offset
    );
    source.seek(SeekFrom::Start(aligned_offset))?;
    Ok(())
}

fn read_region<B: binread::BinRead + Debug>(
    source: &mut (impl Seek + Read),
    offset_in_firmware: u64,
) -> Result<B> {
    source.seek(SeekFrom::Start(offset_in_firmware))?;
    trace!(
        "Trying to parse {} at {}",
        type_name::<B>(),
        offset_in_firmware
    );
    let region = source.read_le::<B>();
    if region.is_err() {
        trace!(
            "Failed to parse region at {}: {:?}",
            offset_in_firmware,
            region
        );
        source.seek(SeekFrom::Start(offset_in_firmware))?;
    }
    Ok(region?)
}

pub trait FirmwareRegion: Debug {
    fn offset_in_firmware(&self) -> u64;

    fn end_offset_in_firmware(&self) -> u64 {
        self.offset_in_firmware() + self.region_size()
    }

    fn region_size(&self) -> u64;
}

pub struct RegionIterator<'a, S: Read + Seek> {
    source: &'a mut S,
}

impl<'a, S: Read + Seek> RegionIterator<'a, S> {
    pub fn new(source: &'a mut S) -> Self {
        Self { source }
    }

    pub fn try_next(&mut self) -> Result<Option<Region>> {
        let mut buf = [0u8; FIRMWARE_REGION_ALIGN as usize];

        align(&mut self.source, FIRMWARE_REGION_ALIGN)?;
        while let Ok(_) = self.source.read_exact(&mut buf) {
            self.source
                .seek(SeekFrom::Current(-(FIRMWARE_REGION_ALIGN as i64)))?;
            let offset_in_firmware = self.source.stream_position()?;
            let signature_2 = &buf[0..2];
            let signature_4 = &buf[0..4];

            trace!(
                "Testing region at {} for 2-bytes signature: {:02X?}",
                offset_in_firmware,
                signature_2
            );
            match signature_2 {
                pci_legacy::PCI_EXPANSION_ROM_HEADER_IDENTIFIER => {
                    if let Ok(region) = read_region::<pci_efi::EfiPciExpansionRom>(
                        &mut self.source,
                        offset_in_firmware,
                    ) {
                        return Ok(Some(Region::EfiPciExpansionRom(region)));
                    }
                    if let Ok(region) = read_region::<pci_legacy::PciExpansionRom>(
                        &mut self.source,
                        offset_in_firmware,
                    ) {
                        return Ok(Some(Region::LegacyPciExpansionRom(region)));
                    }
                }
                nvidia::NV_ROM_SIGNATURE => {
                    if let Ok(region) = read_region::<nvidia::nbsi::NbsiPciExpansionRom>(
                        &mut self.source,
                        offset_in_firmware,
                    ) {
                        return Ok(Some(Region::NbsiPciExpansionRom(region)));
                    }
                    if let Ok(region) = read_region::<nvidia::NvidiaPciExpansionRom>(
                        &mut self.source,
                        offset_in_firmware,
                    ) {
                        return Ok(Some(Region::NvidiaPciExpansionRom(region)));
                    }
                }
                _ => {
                    trace!(
                        "No matches found at {} for 2-bytes signature: {:02X?}",
                        offset_in_firmware,
                        signature_2
                    );
                }
            }

            trace!(
                "Testing region at {} for 4-bytes signature: {:02X?}",
                offset_in_firmware,
                signature_4
            );
            match signature_4 {
                nvidia::NVGI_SIGNATURE => {
                    if let Ok(region) =
                        read_region::<nvidia::NvgiRegion>(&mut self.source, offset_in_firmware)
                    {
                        return Ok(Some(Region::NvgiRegion(region)));
                    }
                }
                nvidia::RFRD_SIGNATURE => {
                    if let Ok(region) =
                        read_region::<nvidia::RfrdRegion>(&mut self.source, offset_in_firmware)
                    {
                        return Ok(Some(Region::RfrdRegion(region)));
                    }
                }
                _ => {
                    trace!(
                        "No matches found at {} for 4-bytes signature: {:02X?}",
                        offset_in_firmware,
                        signature_4
                    );
                }
            }
            self.source
                .seek(SeekFrom::Start(offset_in_firmware + FIRMWARE_REGION_ALIGN))?;
        }

        Ok(None)
    }
}

impl<'a, S: Read + Seek> Iterator for RegionIterator<'a, S> {
    type Item = Region;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().ok().flatten()
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum Region {
    LegacyPciExpansionRom(pci_legacy::PciExpansionRom),
    EfiPciExpansionRom(pci_efi::EfiPciExpansionRom),
    NvidiaPciExpansionRom(nvidia::NvidiaPciExpansionRom),
    NbsiPciExpansionRom(nvidia::nbsi::NbsiPciExpansionRom),
    NvgiRegion(nvidia::NvgiRegion),
    RfrdRegion(nvidia::RfrdRegion),
}

impl Region {
    fn firmware_region(&self) -> &dyn FirmwareRegion {
        match self {
            Region::LegacyPciExpansionRom(region) => region,
            Region::EfiPciExpansionRom(region) => region,
            Region::NvidiaPciExpansionRom(region) => region,
            Region::NbsiPciExpansionRom(region) => region,
            Region::NvgiRegion(region) => region,
            Region::RfrdRegion(region) => region,
        }
    }
}

impl FirmwareRegion for Region {
    fn offset_in_firmware(&self) -> u64 {
        self.firmware_region().offset_in_firmware()
    }

    fn region_size(&self) -> u64 {
        self.firmware_region().region_size()
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum RegionStructure {
    BiosInformationTable(bit::BITStructure),
    DeviceControlBlock(dcb::DeviceControlBlock),
}

pub struct RegionStructureIterator<'a, S: Read + Seek> {
    source: &'a mut S,
}

impl<'a, S: Read + Seek> RegionStructureIterator<'a, S> {
    pub fn new(source: &'a mut S) -> Self {
        Self { source }
    }

    pub fn try_next(&mut self) -> Result<Option<RegionStructure>> {
        let mut buf = [0u8; FIRMWARE_REGION_STRUCTURE_ALIGN as usize * 16];

        trace!("Iterating over structures in region.");
        align(&mut self.source, FIRMWARE_REGION_STRUCTURE_ALIGN)?;
        while let Ok(_) = self.source.read_exact(&mut buf) {
            self.source.seek(SeekFrom::Current(-(buf.len() as i64)))?;
            let offset_in_firmware = self.source.stream_position()?;
            trace!(
                "Testing region at {} for region structures: {:02X?}",
                offset_in_firmware,
                buf
            );
            if &buf[2..6] == bit::BIT_SIGNATURE {
                if let Ok(bit_structure) =
                    read_region::<bit::BITStructure>(&mut self.source, offset_in_firmware)
                {
                    return Ok(Some(RegionStructure::BiosInformationTable(bit_structure)));
                }
            }
            if &buf[6..10] == dcb::DCB_SIGNATURE {
                if let Ok(dcb_structure) =
                    read_region::<dcb::DeviceControlBlock>(&mut self.source, offset_in_firmware)
                {
                    return Ok(Some(RegionStructure::DeviceControlBlock(dcb_structure)));
                }
            }

            self.source.seek(SeekFrom::Start(
                offset_in_firmware + FIRMWARE_REGION_STRUCTURE_ALIGN,
            ))?;
        }

        Ok(None)
    }
}

impl<'a, S: Read + Seek> Iterator for RegionStructureIterator<'a, S> {
    type Item = RegionStructure;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().ok().flatten()
    }
}

#[derive(BinRead, Clone, Copy, Serialize)]
pub struct VersionHex4([u8; 4]);

impl Debug for VersionHex4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02X}.{:02X}.{:02X}.{:02X}",
            self.0[3], self.0[2], self.0[1], self.0[0]
        )
    }
}

impl Display for VersionHex4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl VersionHex4 {
    pub fn non_zero(&self) -> Option<&Self> {
        if self.0 == [0u8; 4] {
            None
        } else {
            Some(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::firmware::FirmwareBundleInfo;
    use log::LevelFilter;
    use reqwest::Url;
    use simplelog::{Config, TestLogger};
    use std::fs::File;
    use std::{env, fs};

    const CACHE_FOLDER: &str = "nv-rom-parser-cache";

    #[test]
    fn test_3060ti() {
        TestLogger::init(LevelFilter::Debug, Config::default()).unwrap();
        let mut rom_file = get_rom_file(
            "https://www.techpowerup.com/vgabios/236055/MSI.RTX3060Ti.8192.201112.rom",
        );
        let firmware_bundle = FirmwareBundleInfo::parse(&mut rom_file).unwrap();
        println!("Firmware: {:#?}", &firmware_bundle);
        println!("\n\n\n{:#?}", firmware_bundle.v_bios_info())
    }

    #[test]
    fn test_3060ti_memory_clock() {
        TestLogger::init(LevelFilter::Debug, Config::default()).unwrap();
        let mut rom_file = get_rom_file(
            "https://www.techpowerup.com/vgabios/236055/MSI.RTX3060Ti.8192.201112.rom",
        );
        let firmware_bundle = FirmwareBundleInfo::parse(&mut rom_file).unwrap();
        if let Some(memory_clock_table) = firmware_bundle
            .firmwares
            .first()
            .and_then(|f| f.legacy_pci_image.as_ref())
            .and_then(|i| i.memory_clock_table.as_ref())
        {
            println!("Memory clock table: {:?}", &memory_clock_table);
            for entry in &memory_clock_table.entries {
                println!("Entry: {:?}", entry.base_entry.unknown)
            }
        }
    }

    #[test]
    fn test_3060ti_memory_tweak() {
        TestLogger::init(LevelFilter::Debug, Config::default()).unwrap();
        let mut rom_file = get_rom_file(
            "https://www.techpowerup.com/vgabios/236055/MSI.RTX3060Ti.8192.201112.rom",
        );
        let firmware_bundle = FirmwareBundleInfo::parse(&mut rom_file).unwrap();
        if let Some(memory_tweak_table) = firmware_bundle
            .firmwares
            .first()
            .and_then(|f| f.legacy_pci_image.as_ref())
            .and_then(|i| i.memory_tweak_table.as_ref())
        {
            //println!("Memory tweak table: {:?}", &memory_tweak_table);
            for entry in &memory_tweak_table.entries {
                println!("Entry: {:?}", entry)
            }
        }
    }

    #[test]
    fn test_4090() {
        TestLogger::init(LevelFilter::Debug, Config::default()).unwrap();
        let mut rom_file = get_rom_file(
            "https://www.techpowerup.com/vgabios/260748/Asus.RTX4090.24576.230321.rom",
        );
        let firmware_bundle = FirmwareBundleInfo::parse(&mut rom_file).unwrap();
        println!("Firmware: {:#?}", &firmware_bundle);
        println!("\n\n\n{:#?}", firmware_bundle.v_bios_info())
    }

    fn get_rom_file(url: &str) -> File {
        let cache_dir = env::temp_dir().join(CACHE_FOLDER);
        let url = Url::parse(url).unwrap();
        let filename = url.path_segments().unwrap().last().unwrap();
        let path = cache_dir.join(filename);

        if !cache_dir.exists() {
            fs::create_dir(cache_dir).unwrap();
        }
        if !path.exists() {
            let mut response = reqwest::blocking::get(url).unwrap();
            let mut file = File::create(&path).unwrap();
            response.copy_to(&mut file).unwrap();
        }
        File::open(&path).unwrap()
    }
}
