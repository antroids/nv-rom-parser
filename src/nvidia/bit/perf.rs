use super::PerfPtrsToken;
use binread::BinRead;
use modular_bitfield::bitfield;
use modular_bitfield::prelude::*;
use serde::Serialize;
use std::io::SeekFrom;

// #[derive(BinRead, Debug, Clone, Serialize)] todo
// pub struct FanCoolerTable {
//     pub version: u8,
//     pub header_size: u8,
//     pub entry_size: u8,
//     pub entry_count: u8,
//     pub unk_1: u8,
//     pub unk_2: u8,
// }
//
// #[derive(BinRead, Debug, Clone, Serialize)]
// pub struct ThermalDeviceTable {
//     pub header: ThermalDeviceTableHeader,
//     #[br(count(header.entry_count))]
//     pub entries: Vec<ThermalDeviceTableEntry>,
// }
//
// #[derive(BinRead, Debug, Clone, Serialize)]
// pub struct ThermalDeviceTableHeader {
//     pub version: u8,
//     #[br(assert(header_size == 4))]
//     pub header_size: u8,
//     pub entry_count: u8,
//     pub entry_size: u8,
// }
//
// #[derive(BinRead, Debug, Clone, Serialize)]
// pub struct ThermalDeviceTableEntry {
//     pub unk: [u8; 11],
// }

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(ptrs: PerfPtrsToken))]
pub struct MemoryClockTable {
    #[br(seek_before = SeekFrom::Start(ptrs.memory_clock_table_ptr as u64))]
    pub header: MemoryClockTableHeader,
    #[br(count(header.entry_count))]
    #[br(args(header.base_entry_size, header.strap_entry_size, header.strap_entry_count))]
    pub entries: Vec<MemoryClockTableEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(packed)]
pub struct MemoryClockTableHeader {
    //#[br(assert(version == 0x20))]
    pub version: u8,
    #[br(assert(header_size == 26))]
    pub header_size: u8,
    pub base_entry_size: u8,   // 86
    pub strap_entry_size: u8,  // 44
    pub strap_entry_count: u8, // 14
    pub entry_count: u8,       // 10
    pub unknown: [u8; 20],
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(base_entry_size: u8, strap_entry_size: u8, strap_entry_count: u8))]
pub struct MemoryClockTableEntry {
    #[br(args(base_entry_size))]
    pub base_entry: MemoryClockTableBaseEntry,
    #[br(count(strap_entry_count))]
    #[br(args(strap_entry_size))]
    pub strap_entries: Vec<MemoryClockTableStrapEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(base_entry_size: u8))]
pub struct MemoryClockTableBaseEntry {
    #[br(map(|v: u16| v & 0x3F))]
    pub min_freq: u16,
    #[br(map(|v: u16| v & 0x3F))]
    pub max_freq: u16,
    pub reserved: [u8; 4],

    #[br(count(base_entry_size - 8))]
    pub unknown: Vec<u8>, // todo
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(strap_entry_size: u8))]
pub struct MemoryClockTableStrapEntry {
    pub mem_tweak_index: u8,
    pub flags_0: u8,
    pub reserved_0: [u8; 6],
    pub flags_4: u8,
    pub reserved_1: u8,
    pub flags_5: u8,

    #[br(count(strap_entry_size - 11))]
    pub unknown: Vec<u8>, //todo
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(ptrs: PerfPtrsToken))]
pub struct PowerPolicyTable {
    #[br(seek_before = SeekFrom::Start(ptrs.power_policy_table_ptr as u64))]
    pub header: PowerPolicyTableHeader,
    #[br(seek_before = SeekFrom::Start(ptrs.power_policy_table_ptr as u64 + header.header_size as u64))]
    #[br(count(header.entry_count))]
    pub entries: Vec<PowerPolicyTableEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct PowerPolicyTableHeader {
    #[br(assert(version == 0x30))]
    pub version: u8,
    pub header_size: u8,
    pub entry_size: u8,
    pub entry_count: u8,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct PowerPolicyTableEntry {
    pub unk_0: u16,
    pub min: u32,
    pub avg: u32,
    pub peak: u32,
    pub unk_1: u32,
    #[br(count(49))]
    pub unk_2: Vec<u8>,
}

// https://nvidia.github.io/open-gpu-doc/virtual-p-state-table/virtual-P-state-table.html
// https://docs.nvidia.com/gameworks/content/gameworkslibrary/coresdk/nvapi/group__gpupstate.html
#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(ptrs: PerfPtrsToken))]
pub struct VirtualPStateTable20 {
    #[br(seek_before = SeekFrom::Start(ptrs.virtual_p_state_table_ptr as u64))]
    pub header: VirtualPStateTableHeader20,
    #[br(count(header.entry_count))]
    #[br(args(header.domain_freq_entry_count))]
    pub entries: Vec<VirtualPStateTableEntry20>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct VirtualPStateTableHeader20 {
    #[br(assert(version == 0x20))]
    pub version: u8,
    pub header_size: u8,
    #[br(assert(base_entry_size == 1))]
    pub base_entry_size: u8,
    pub entry_count: u8,
    #[br(assert(domain_freq_entry_size == 4))]
    pub domain_freq_entry_size: u8,
    pub domain_freq_entry_count: u8,

    // P0/P1 - Maximum 3D performance
    // P2/P3 - Balanced 3D performance-power
    // P8 - Basic HD video playback
    // P10 - DVD playback
    // P12 - Minimum idle power consumption
    // OR
    // boost_entry
    // turbo_boost_entry
    // rated_tdp_entry
    // vrhot_entry
    // max_batt_entry
    // unk15_entry
    // unk16_entry
    #[br(count(header_size - 6))]
    pub p_state_indexes: Vec<u8>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(domain_freq_entry_count: u8))]
pub struct VirtualPStateTableEntry20 {
    pub p_state: u8,
    // Domains probably:
    // nv clock
    // mem clock
    // mem transfer clock
    // processor clock
    // unknown
    #[br(count(domain_freq_entry_count as usize))]
    pub domains_entries: Vec<VirtualPStateTableDomainEntry20>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct VirtualPStateTableDomainEntry20 {
    #[br(restore_position)]
    #[br(map(|v: u8| [v & 0x8 > 0, v & 0x4 > 0]))]
    pub flags_1: [bool; 2],
    #[br(map(|v: u16| (v & 0x3FFF) as u32))]
    pub frequency_1: u32,
    #[br(restore_position)]
    #[br(map(|v: u8| [v & 0x8 > 0, v & 0x4 > 0]))]
    pub flags_2: [bool; 2],
    #[br(map(|v: u16| (v << 2) as u32))]
    pub frequency_2: u32,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(ptrs: PerfPtrsToken))]
pub struct MemoryTweakTable {
    #[br(seek_before = SeekFrom::Start(ptrs.memory_tweak_table_ptr as u64))]
    pub header: MemoryTweakTableHeader,
    #[br(count(header.entry_count))]
    #[br(args(header.extended_entry_count))]
    pub entries: Vec<MemoryTweakTableEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct MemoryTweakTableHeader {
    #[br(assert(version == 0x20))]
    pub version: u8,
    #[br(assert(header_size == 6))]
    pub header_size: u8,
    #[br(assert(base_entry_size == 76))]
    pub base_entry_size: u8,
    #[br(assert(extended_entry_size == 12))]
    pub extended_entry_size: u8,
    pub extended_entry_count: u8,
    pub entry_count: u8,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(extended_entry_count: u8))]
pub struct MemoryTweakTableEntry {
    pub base_entry: MemoryTweakTableBaseEntry,
    #[br(count(extended_entry_count))]
    pub extended_entries: Vec<MemoryTweakTableExtendedEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct MemoryTweakTableBaseEntry {
    pub config_0: MemoryTweakTableBaseEntryConfig0,
    pub config_1: MemoryTweakTableBaseEntryConfig1,
    pub config_2: MemoryTweakTableBaseEntryConfig2,
    pub config_3: MemoryTweakTableBaseEntryConfig3,
    pub config_4: MemoryTweakTableBaseEntryConfig4,
    pub config_5: MemoryTweakTableBaseEntryConfig5,

    pub reserved_0: [u8; 23],

    pub voltage_config: MemoryTweakTableBaseEntryVoltageConfig, // 9 bytes
    pub timing_config: MemoryTweakTableBaseEntryTiming22,

    pub reserved_1: [u8; 16],
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize, BitfieldSpecifier)]
pub struct MemoryTweakTableBaseEntryConfig0 {
    pub rc: u8,
    pub rfc: B9,
    pub ras: B7,
    pub rp: B7,
    pub reserved_0: B1,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize, BitfieldSpecifier)]
pub struct MemoryTweakTableBaseEntryConfig1 {
    pub cl: B7,
    pub wl: B7,
    pub rd_rcd: B6,
    pub wr_rcd: B6,
    pub reserved_1: B6,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize, BitfieldSpecifier)]
pub struct MemoryTweakTableBaseEntryConfig2 {
    pub rpre: B4,
    pub wpre: B4,
    pub cdlr: B7,
    pub reserved_3: B1,
    pub wr: B7,
    pub reserved_4: B1,
    pub w2r_bus: B4,
    pub r2w_bus: B4,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize, BitfieldSpecifier)]
pub struct MemoryTweakTableBaseEntryConfig3 {
    pub pdex: B5,
    pub pden2pdex: B4,
    pub faw: u8,
    pub aond: B7,
    pub ccdl: B4,
    pub ccds: B4,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize, BitfieldSpecifier)]
pub struct MemoryTweakTableBaseEntryConfig4 {
    pub refresh_lo: B3,
    pub refresh: B12,
    pub rrd: B6,
    pub delay_0: B6,
    pub reserved_5: B5,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize, BitfieldSpecifier)]
pub struct MemoryTweakTableBaseEntryConfig5 {
    pub adr_min: B3,
    pub reserved_6: B1,
    pub wrcrc: B7,
    pub reserved_7: B1,
    pub offset_0: B6,
    pub delay_0_msb: B2,
    pub offset_1: B4,
    pub offset_2: B4,
    pub delay_0_1: B4,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize, BitfieldSpecifier)]
pub struct MemoryTweakTableBaseEntryVoltageConfig {
    pub drive_strength: B2,
    pub voltage_0: B3,
    pub voltage_1: B3,

    pub voltage_2: B3,
    pub r2p: B5,

    pub voltage_3: B3,
    pub reserved_0: B1,
    pub voltage_4: B3,
    pub reserved_1: B1,

    pub voltage_5: B3,
    pub reserved_2: B5,

    pub rdcrc: B4,
    pub reserved_3: B36,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize, BitfieldSpecifier)]
pub struct MemoryTweakTableBaseEntryTiming22 {
    pub rfcsba: B10,
    pub rfcsbr: B8,
    pub reserved: B14,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct MemoryTweakTableExtendedEntry {
    #[br(count(12))]
    pub unknown: Vec<u8>,
}
