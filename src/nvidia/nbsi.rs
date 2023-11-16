// SPDX-License-Identifier: MIT

use crate::pci_legacy::PciExpansionRomDataHeader;
use crate::{FirmwareRegion, FIRMWARE_REGION_ALIGN};
use binread::BinRead;
use derivative::Derivative;
use serde::Serialize;
use std::fmt::{Debug, Formatter};
use std::mem::size_of;
use strum::FromRepr;

pub const NBSI_SIGNATURE: &[u8] = b"ISBN";

// https://github.com/NVIDIA/open-gpu-kernel-modules/blob/main/src/nvidia/inc/kernel/platform/pci_exp_table.h
// https://github.com/NVIDIA/open-gpu-kernel-modules/blob/main/src/nvidia/inc/kernel/platform/nbsi/nbsi_table.h
#[derive(BinRead, Derivative, Clone, Serialize)]
#[derivative(Debug)]
pub struct NbsiPciExpansionRom {
    #[br(align_before = FIRMWARE_REGION_ALIGN)]
    #[br(parse_with = crate::stream_position)]
    pub offset_in_firmware: u64,
    pub header: NbsiPciExpansionRomHeader,
    #[br(seek_before = binread::io::SeekFrom::Start(offset_in_firmware + header.pcir_offset as u64))]
    #[br(assert(data_header.signature == crate::nvidia::NV_PCI_DATA_STRUCTURE_SIGNATURE))]
    pub data_header: PciExpansionRomDataHeader,
    #[br(align_before = 16)]
    #[br(try)]
    pub data_header_extended: Option<crate::nvidia::NvidiaPciDataExtended>,
    #[br(seek_before = binread::io::SeekFrom::Start(offset_in_firmware + header.nbsi_data_offset as u64))]
    pub nbsi_directory: NbsiDirectory,
}

impl FirmwareRegion for NbsiPciExpansionRom {
    fn offset_in_firmware(&self) -> u64 {
        self.offset_in_firmware
    }

    fn region_size(&self) -> u64 {
        self.data_header.image_length as u64 * 512
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NbsiPciExpansionRomHeader {
    #[br(assert(signature == crate::nvidia::NV_ROM_SIGNATURE))]
    pub signature: [u8; 2],
    pub _reserved: [u8; 20],
    pub nbsi_data_offset: u16,
    pub pcir_offset: u16,
    pub nbsi_block_size: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NbsiDirectory {
    #[br(parse_with = crate::stream_position)]
    pub offset_in_region: u64,
    #[br(assert(signature == NBSI_SIGNATURE))]
    pub signature: [u8; 4],
    pub size: u32,
    pub globals_count: u8,
    pub driver: u8,
    #[br(count(globals_count))]
    pub objects_global_types: Vec<NbsiGlobal>,
    #[br(parse_with = crate::stream_position)]
    pub objects_offset_in_region: u64,
    #[br(count(globals_count))]
    pub objects: Vec<NbsiGenericObject>,
}

#[derive(BinRead, Clone, Serialize)]
pub struct NbsiGlobal(u16);

impl NbsiGlobal {
    pub fn global_type(&self) -> Option<GlobalType> {
        GlobalType::from_repr(self.0)
    }
}

impl Debug for NbsiGlobal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NbsiGlobal")
            .field("raw", &self.0)
            .field("global_type", &self.global_type())
            .finish()
    }
}

#[derive(BinRead, Debug, Clone, Serialize, FromRepr)]
#[repr(u16)]
#[br(repr = u16)]
pub enum GlobalType {
    Reserved = 0x00,
    Driver = u16::from_le_bytes(*b"DR"),
    VBios = u16::from_le_bytes(*b"VB"),
    Hdcp = u16::from_le_bytes(*b"HK"),
    InfoRom = u16::from_le_bytes(*b"IR"),
    Hdd = u16::from_le_bytes(*b"HD"),
    NonVolatile = u16::from_le_bytes(*b"NV"),
    PlatInfo = u16::from_le_bytes(*b"PI"),
    PlatInfoWar = u16::from_le_bytes(*b"IP"),
    ValKey = u16::from_le_bytes(*b"VK"),
    TegraInfo = u16::from_le_bytes(*b"TG"),
    TegraDcb = u16::from_le_bytes(*b"TD"),
    TegraPanel = u16::from_le_bytes(*b"TP"),
    TegraDsi = u16::from_le_bytes(*b"TS"),
    SysInfo = u16::from_le_bytes(*b"GD"),
    TegraTmds = u16::from_le_bytes(*b"TT"),
    OptimusPlat = u16::from_le_bytes(*b"OP"),
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NbsiGenericObject {
    #[br(parse_with = crate::stream_position)]
    pub offset_in_region: u64,
    pub header: NbsiGenericObjectHeader,
    #[br(calc(header.size as u64 - size_of::<NbsiGenericObjectHeader>() as u64))]
    pub data_size: u64,
    #[br(parse_with = crate::stream_position)]
    #[br(pad_after(data_size as i64))]
    pub data_offset_in_region: u64,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(packed)]
pub struct NbsiGenericObjectHeader {
    pub hash_signature: u64,
    pub global_type: u16,
    pub size: u32,
    pub min_version: u8,
    pub max_version: u8,
}
