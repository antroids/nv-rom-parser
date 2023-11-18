use super::PerfPtrsToken;
use binread::BinRead;
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
