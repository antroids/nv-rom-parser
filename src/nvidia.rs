// SPDX-License-Identifier: MIT

use crate::pci_legacy::{PciExpansionRomDataHeader, PciExpansionRomIndicator};
use crate::{FirmwareRegion, VersionHex4, FIRMWARE_REGION_ALIGN};
use binread::BinRead;
use bitflags::bitflags;
use derivative::Derivative;
use serde::Serialize;

pub mod bit;
pub mod dcb;
pub mod nbsi;

pub const NV_ROM_SIGNATURE: &[u8] = b"VN";

pub const NVGI_SIGNATURE: &[u8] = b"NVGI";
pub const RFRD_SIGNATURE: &[u8] = b"RFRD";

const NV_PCI_DATA_STRUCTURE_SIGNATURE: &[u8] = b"NPDS";
const NV_PCI_DATA_EXTENDED_STRUCTURE_SIGNATURE: &[u8] = b"NPDE";

#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(packed)]
pub struct NvgiHeader {
    #[br(assert(signature == NVGI_SIGNATURE))]
    pub signature: [u8; 4],
    pub unknown1: u16,
    pub unknown2: u16,
    pub size: u32,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NvgiRegion {
    #[br(align_before = FIRMWARE_REGION_ALIGN)]
    #[br(parse_with = crate::stream_position)]
    pub offset_in_firmware: u64,
    pub header: NvgiHeader,
    #[br(calc(header.size as u64))]
    pub data_size: u64,
    #[br(pad_after(data_size as i64))]
    #[br(parse_with = crate::stream_position)]
    pub data_offset_in_firmware: u64,
}

impl FirmwareRegion for NvgiRegion {
    fn offset_in_firmware(&self) -> u64 {
        self.offset_in_firmware
    }

    fn region_size(&self) -> u64 {
        self.data_size
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(packed)]
pub struct RfrdHeader {
    #[br(assert(signature == RFRD_SIGNATURE))]
    pub signature: [u8; 4],
    pub unknown1: u16,
    pub unknown2: u16,
    pub pci_rom_offset: u32,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct RfrdRegion {
    #[br(align_before = FIRMWARE_REGION_ALIGN)]
    #[br(parse_with = crate::stream_position)]
    pub offset_in_firmware: u64,
    pub header: RfrdHeader,
}

impl FirmwareRegion for RfrdRegion {
    fn offset_in_firmware(&self) -> u64 {
        self.offset_in_firmware
    }

    fn region_size(&self) -> u64 {
        16
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NvidiaPciDataExtended {
    #[br(assert(signature == NV_PCI_DATA_EXTENDED_STRUCTURE_SIGNATURE))]
    pub signature: [u8; 4],
    pub revision: u16,
    pub structure_length: u16,
    pub image_length: u16,
    pub indicator: PciExpansionRomIndicator,
    pub flags: NvidiaPciDataExtendedFlags,
    #[br(if(structure_length > 12))]
    pub gop_version: Option<VersionHex4>,
    #[br(if(structure_length > 14))]
    pub subsystem_id: Option<VersionHex4>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NvidiaPciDataExtendedFlags(u8);
bitflags! {
    impl NvidiaPciDataExtendedFlags: u8 {
        const PrivateImagesEnabled = 0b00000001;
    }
}

#[derive(BinRead, Derivative, Clone, Serialize)]
#[derivative(Debug)]
pub struct NvidiaPciExpansionRom {
    #[br(align_before = FIRMWARE_REGION_ALIGN)]
    #[br(parse_with = crate::stream_position)]
    pub offset_in_firmware: u64,
    pub header: NvidiaPciExpansionRomHeader,
    #[br(seek_before = binread::io::SeekFrom::Start(offset_in_firmware + header.pcir_offset as u64))]
    #[br(assert(data_header.signature == NV_PCI_DATA_STRUCTURE_SIGNATURE))]
    pub data_header: PciExpansionRomDataHeader,
    #[br(align_before = 16)]
    #[br(try)]
    pub data_header_extended: Option<NvidiaPciDataExtended>,
    #[br(seek_before = binread::io::SeekFrom::Start(offset_in_firmware))]
    #[br(count(data_header.image_length))]
    #[derivative(Debug = "ignore")]
    #[serde(skip)]
    pub data: Vec<u8>,
}

impl FirmwareRegion for NvidiaPciExpansionRom {
    fn offset_in_firmware(&self) -> u64 {
        self.offset_in_firmware
    }

    fn region_size(&self) -> u64 {
        self.data_header.image_length as u64 * 512
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NvidiaPciExpansionRomHeader {
    #[br(assert(signature == NV_ROM_SIGNATURE))]
    pub signature: [u8; 2],
    pub _reserved: [u8; 22],
    pub pcir_offset: u16,
}
