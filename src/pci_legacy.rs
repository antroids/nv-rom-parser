// SPDX-License-Identifier: MIT

use crate::nvidia::NvidiaPciDataExtended;
use crate::{FirmwareRegion, FIRMWARE_REGION_ALIGN};
use binread::BinRead;
use derivative::Derivative;
use serde::Serialize;

pub const PCI_EXPANSION_ROM_HEADER_IDENTIFIER: &[u8] = b"\x55\xAA";
pub const PCI_EXPANSION_ROM_DATA_IDENTIFIER: &[u8] = b"PCIR";

#[derive(BinRead, Derivative, Clone, Serialize)]
#[derivative(Debug)]
pub struct PciExpansionRom {
    #[br(align_before = FIRMWARE_REGION_ALIGN)]
    #[br(parse_with = crate::stream_position)]
    pub offset_in_firmware: u64,
    pub header: PciExpansionRomHeader,
    #[br(seek_before = binread::io::SeekFrom::Start(offset_in_firmware + header.pcir_offset as u64))]
    #[br(assert(data_header.signature == PCI_EXPANSION_ROM_DATA_IDENTIFIER))]
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

impl FirmwareRegion for PciExpansionRom {
    fn offset_in_firmware(&self) -> u64 {
        self.offset_in_firmware
    }

    fn region_size(&self) -> u64 {
        self.data_header.image_length as u64 * 512
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct PciExpansionRomHeader {
    #[br(assert(signature == PCI_EXPANSION_ROM_HEADER_IDENTIFIER))]
    pub signature: [u8; 2],
    pub initialization_size: u8, // x512
    pub init_function_ptr: [u8; 3],
    pub _reserved: [u8; 18], // 12 in PCI rev 2.2 standard
    pub pcir_offset: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct PciExpansionRomDataHeader {
    pub signature: [u8; 4],
    pub vendor_id: u16,
    pub device_id: u16,
    pub device_list_ptr: u16,
    pub pci_data_structure_length: u16,
    pub pci_data_structure_revision: u8,
    pub class_code: [u8; 3],
    pub image_length: u16,
    pub revision_level: u16,
    pub code_type: PciExpansionRomCodeType,
    pub indicator: PciExpansionRomIndicator,
    pub max_runtime_image_length: u16,
    pub configuration_utility_code_pointer: u16,
    pub dmtf_clp_entry_point_pointer: u16,
} // 28 bytes

#[derive(BinRead, Debug, Clone, Serialize, PartialEq)]
#[repr(u8)]
#[br(repr = u8)]
pub enum PciExpansionRomCodeType {
    Ia32PcAtCompatible = 0x0,
    OpenFirmwareStandardForPci = 0x1,
    HewlettPackardPaRisc = 0x2,
    EfiImage = 0x3,
    NvidiaX86Extension = 0xe0,
    NvidiaHDCP = 0x85,
    NvidiaNbsiSignature = 0x70,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(u8)]
#[br(repr = u8)]
pub enum PciExpansionRomIndicator {
    AnotherImageFollows = 0b00000000,
    LastImage = 0b010000000,
}
