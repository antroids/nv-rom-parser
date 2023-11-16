// SPDX-License-Identifier: MIT

use binread::BinRead;
use bitflags::bitflags;
use modular_bitfield::prelude::{B1, B2, B4};
use modular_bitfield::{bitfield, BitfieldSpecifier};
use serde::Serialize;
use std::io::SeekFrom;

bitflags! {
    impl NvLinkVbiosParam4TxtrainOptimizatopnAlgorithm: u8 {
        const Rsvd = 0x00;
        const A0SinglePresent = 0x01;
        const A1PresentArray = 0x02;
        const A2FineGrainedExhaustive = 0x04;
        const A3Rsvd = 0x08;
        const A4FomCentriod = 0x10;
        const A5Rsvd = 0x20;
        const A6Rsvd = 0x40;
        const A7Rsvd = 0x80;
    }
}
bitflags! {
    impl NvLinkVbiosParam5Txtrain: u8 {
        const AdjustmentAlgorithmB0NoAdjustment = 0x10;
        const AdjustmentAlgorithmB1FixedAdjustment = 0x20;
        const AdjustmentAlgorithmB2Rsvd = 0x40;
        const AdjustmentAlgorithmB3Rsvd = 0x80;

        const FomFormatFomA = 0x01;
        const FomFormatFomB = 0x02;
        const FomFormatFomC = 0x04;
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(ptrs: super::NvinitPtrsToken))]
pub struct NvLinkConfigData {
    #[br(seek_before = SeekFrom::Start(ptrs.nvlink_configuration_data_ptr as u64))]
    pub header: NvLinkConfigDataHeader,
    #[br(count(header.base_entry_count))]
    #[br(args(header.link_entry_count, header.link_entry_size))]
    pub entries: Vec<NvLinkEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NvLinkConfigDataHeader {
    pub version: u8,
    #[br(assert(header_size == 8))]
    pub header_size: u8,
    #[br(assert(base_entry_size == 1))]
    pub base_entry_size: u8,
    pub base_entry_count: u8,
    pub link_entry_size: u8,
    pub link_entry_count: u8,
    pub reserved: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(link_entry_count: u8, link_entry_size: u8))]
pub struct NvLinkEntry {
    pub position_id: u8,
    #[br(count(link_entry_count))]
    #[br(args(link_entry_size))]
    pub link_entries: Vec<NvLinkLinkEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(link_entry_size: u8))]
pub struct NvLinkLinkEntry {
    pub param_0: NvLinkVbiosParam0,
    pub param_1: NvLinkVbiosParam1,
    pub param_2: NvLinkVbiosParam2,
    pub param_3: NvLinkVbiosParam3,
    pub param_4: NvLinkVbiosParam4TxtrainOptimizatopnAlgorithm,
    pub param_5: NvLinkVbiosParam5Txtrain,
    pub param_6: NvLinkVbiosParam6TxtrainMinimumTrainTime,
    #[br(count(link_entry_size - 7))]
    pub extra_params: Vec<u8>,
}

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
#[br(map = Self::from_bytes)]
pub struct NvLinkVbiosParam0 {
    pub link: bool,
    pub reserved_1: B1,
    pub ac_mode: bool,
    pub receiver_detect_enable: bool,
    pub restore_phy_training_enable: bool,
    pub slm_enable: bool,
    pub l2_enable: bool,
    pub reserved_2: B1,
}

#[derive(Copy, Clone, Debug, BinRead, Serialize)]
#[repr(u8)]
#[br(repr = u8)]
pub enum NvLinkVbiosParam1 {
    LineRate5_000_000,
    LineRate1_600_000,
    LineRate2_000_000,
    LineRate2_500_000,
    LineRate2_578_125,
    LineRate3_200_000,
    LineRate4_000_000,
    LineRate5_312_500,
    Unknown0x08,
}

#[derive(Copy, Clone, Debug, BinRead, Serialize)]
#[repr(u8)]
#[br(repr = u8)]
pub enum NvLinkVbiosParam2 {
    CodeModeNrz,
    CodeModeNrz128B130,
    CodeModeNrzPam4,
}

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
pub struct NvLinkVbiosParam3 {
    pub reference_clock_mode: ReferenceClockMode,
    pub reserved_1: B2,
    pub clock_mode_block_mode: ClockModeBlockCode,
    pub reserved_2: B2,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NvLinkVbiosParam4TxtrainOptimizatopnAlgorithm(u8);

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct NvLinkVbiosParam5Txtrain(u8);

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
pub struct NvLinkVbiosParam6TxtrainMinimumTrainTime {
    pub mantissa: B4,
    pub exponent: B4,
}

#[derive(Debug, Copy, Clone, PartialEq, BitfieldSpecifier, Serialize)]
#[bits = 2]
pub enum ReferenceClockMode {
    Common,
    Rsvd,
    NonCommonNoSs,
    NonCommonSs,
}

#[derive(Debug, Copy, Clone, PartialEq, BitfieldSpecifier, Serialize)]
#[bits = 2]
pub enum ClockModeBlockCode {
    Off,
    Ecc96,
    Ecc88,
}
