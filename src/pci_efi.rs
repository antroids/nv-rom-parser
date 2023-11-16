// SPDX-License-Identifier: MIT

use crate::nvidia::NvidiaPciDataExtended;
use crate::pci_legacy::{
    PciExpansionRomDataHeader, PCI_EXPANSION_ROM_DATA_IDENTIFIER,
    PCI_EXPANSION_ROM_HEADER_IDENTIFIER,
};
use crate::{FirmwareRegion, FIRMWARE_REGION_ALIGN};
use binread::io::SeekFrom;
use binread::BinRead;
use derivative::Derivative;
use serde::Serialize;

const EFI_SIGNATURE: &[u8] = b"\xf1\x0e\0\0";

#[derive(BinRead, Derivative, Clone, Serialize)]
#[derivative(Debug)]
pub struct EfiPciExpansionRom {
    #[br(align_before = FIRMWARE_REGION_ALIGN)]
    #[br(parse_with = crate::stream_position)]
    pub offset_in_firmware: u64,
    pub header: EfiPciExpansionRomHeader,
    #[br(seek_before = SeekFrom::Start(header.pcir_offset as u64 + offset_in_firmware))]
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

impl FirmwareRegion for EfiPciExpansionRom {
    fn offset_in_firmware(&self) -> u64 {
        self.offset_in_firmware
    }

    fn region_size(&self) -> u64 {
        self.data_header.image_length as u64 * 512
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct EfiPciExpansionRomHeader {
    #[br(assert(signature == PCI_EXPANSION_ROM_HEADER_IDENTIFIER))]
    pub signature: [u8; 2],
    pub initialization_size: u16, // x512
    #[br(assert(efi_signature == EFI_SIGNATURE))]
    pub efi_signature: [u8; 4],
    pub efi_subsystem: EfiPciExpansionRomSubsystem,
    pub efi_machine_type: EfiPciExpansionRomMachineType,
    pub compression_type: EfiPciExpansionRomCompression,
    pub _reserved: [u8; 8],
    pub efi_image_header_offset: u16,
    pub pcir_offset: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(u16)]
#[br(repr = u16)]
pub enum EfiPciExpansionRomSubsystem {
    BootServiceDriver = 0x0B,
    RuntimeDriver = 0x0C,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(u16)]
#[br(repr = u16)]
pub enum EfiPciExpansionRomMachineType {
    Ia32 = 0x014C,
    Itanium = 0x0200,
    EfiByteCode = 0x0EBC,
    X64 = 0x8664,
    Arm = 0x01c2,
    Arm64 = 0xAA64,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(u16)]
#[br(repr = u16)]
pub enum EfiPciExpansionRomCompression {
    Uncompressed = 0x0,
    UefiCompressionAlgorithm = 0x1,
}
