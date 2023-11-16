// SPDX-License-Identifier: MIT

use crate::Result;
use crate::{Error, VersionHex4};
use binread::{BinRead, BinReaderExt};
use bitflags::bitflags;
use serde::Serialize;
use std::ffi::CStr;
use std::fmt::Debug;
use std::io::{Read, Seek, SeekFrom};

pub mod nvlink;

pub const BIT_SIGNATURE: &[u8] = b"BIT\0";

//const BIT_HEADER_IDENTIFIER: u16 = 0xB8FF;

fn try_map_to_string(bytes: Vec<u8>) -> Option<String> {
    CStr::from_bytes_until_nul(bytes.as_slice())
        .ok()
        .map(|c_str| c_str.to_str().ok())
        .flatten()
        .map(|str| str.to_string())
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct BITStructure {
    pub header: BITHeader,
    #[br(count = header.token_entries)]
    pub tokens: Vec<BITToken>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct BITHeader {
    pub id: u16,
    #[br(assert(signature == BIT_SIGNATURE))]
    pub signature: [u8; 4],
    pub version_minor: u8,
    pub version_major: u8,
    pub header_size: u8,
    pub token_size: u8,
    pub token_entries: u8,
    pub header_checksum: u8,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(little)]
pub struct BITToken {
    pub id: u8,
    pub data_version: u8,
    pub data_size: u16,
    pub data_pointer: u16,
}

impl BITToken {
    pub fn data<S: Seek + Read>(&self, source: &mut S) -> Result<BITTokenType> {
        if self.data_pointer == 0 {
            return Ok(BITTokenType::Nop);
        } else {
            source.seek(SeekFrom::Start(self.data_pointer as u64))?;
            match self.id {
                0x32 => Ok(BITTokenType::I2C(source.read_le()?)),
                0x41 => Ok(BITTokenType::Dac(source.read_le()?)),
                0x42 => Ok(BITTokenType::Bios(source.read_le()?)),
                0x43 => Ok(BITTokenType::Clock(source.read_le()?)),
                0x44 => Ok(BITTokenType::Dfp(source.read_le()?)),
                0x49 => Ok(BITTokenType::NvInit(source.read_le()?)),
                0x4C => Ok(BITTokenType::Lvds(source.read_le()?)),
                0x4D => Ok(BITTokenType::Memory(source.read_le()?)),
                0x4E => Ok(BITTokenType::Nop),
                0x50 => Ok(BITTokenType::Perf(source.read_le()?)),
                0x52 => Ok(BITTokenType::BridgeFw(source.read_le()?)),
                0x53 => Ok(BITTokenType::String(source.read_le()?)),
                0x54 => Ok(BITTokenType::Tmds(source.read_le()?)),
                0x55 => Ok(BITTokenType::Display(source.read_le()?)),
                0x56 => Ok(BITTokenType::Virtual(source.read_le()?)),
                0x63 => Ok(BITTokenType::Ptrs32Bit(source.read_le()?)),
                0x64 => Ok(BITTokenType::Dp(source.read_le()?)),
                0x6E => Ok(BITTokenType::Dcb(source.read_le()?)),
                0x70 => Ok(BITTokenType::Falcon(source.read_le()?)),
                0x75 => Ok(BITTokenType::Uefi(source.read_le()?)),
                0x78 => Ok(BITTokenType::Mxm(source.read_le()?)),
                _ => Err(Error::InvalidFormat(format!(
                    "Unexpected BIT token id: {}",
                    self.id
                ))),
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum BITTokenType {
    I2C(I2CPtrsToken),
    Dac(DACPtrsToken),
    Bios(BiosDataToken),
    Clock(ClockPtrsToken),
    Dfp(DfpPtrsToken),
    NvInit(NvinitPtrsToken),
    Lvds(LvdsPtrsToken),
    Memory(MemoryPtrsToken),
    Nop,
    Perf(PerfPtrsToken),
    BridgeFw(BridgeFwDataToken),
    String(StringPtrsToken),
    Tmds(TmdsPtrsToken),
    Display(DisplayPtrsToken),
    Virtual(VirtualPtrsToken),
    Ptrs32Bit(Vec<u32>),
    Dp(DpPtrsToken),
    Dcb(DcbPtrsToken),
    Falcon(FalconDataToken),
    Uefi(UefiDataToken),
    Mxm(MxmDataToken),
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct I2CPtrsToken {
    pub i2c_scripts_ptr: u16,
    pub ext_hw_mon_init_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct DACPtrsToken {
    pub dac_data_ptr: u16,
    pub dac_flags: DacFlags,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct DacFlags(u8);
bitflags! {
    impl DacFlags: u8 {
        const DacSleepModeSupport = 0b00000001;
    }
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct BiosDataToken {
    pub bios_version: VersionHex4,
    pub bios_oem_version: u8,
    pub bios_checksum: u8,
    pub int15_post_callbacks: Int15PostCallbacks,
    pub int15_system_callbacks: Int15SystemCallbacks,
    pub frame_count: u16,
    pub _reserved: u32,
    pub max_heads_at_post: u8,
    pub memory_size_report: u8,
    pub h_scale_factor: u8,
    pub v_scale_factor: u8,
    pub data_range_table_pointer: u16,
    pub rom_packs_pointer: u16,
    pub applied_rom_packs_pointer: u16,
    pub applied_rom_pack_max: u8,
    pub applied_rom_pack_count: u8,
    pub module_map_external_0: ModuleMapExternal0,
    pub compression_data_table: u32,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct Int15PostCallbacks(u16);
bitflags! {
    impl Int15PostCallbacks: u16 {
        const GetPanelId = 0b00000000_00000001;
        const GetTvFormat = 0b00000000_00000010;
        const GetBootDevice = 0b00000000_00000100;
        const GetPanelExpansionCentering = 0b00000000_00001000;
        const PerformPostCompleteCallback = 0b00000000_00010000;
        const GetRamConfiguration = 0b00000000_00100000;
        const GetTvConnectionType = 0b00000000_01000000;
        const OemExternalInitialization = 0b00000000_10000000;
    }
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct Int15SystemCallbacks(u16);
bitflags! {
    impl Int15SystemCallbacks: u16 {
        const MakeDpmsBypassCallback = 0b00000000_00000001;
        const GetTvFormatCallback = 0b00000000_00000010;
        const MakeSpreadSpectrumBypassCallback = 0b00000000_00000100;
        const MakeDisplaySwitchBypassCallback = 0b00000000_00001000;
        const MakeDeviceControlSettingBypassCallback = 0b00000000_00010000;
        const MakeDdcCallBypassCallback = 0b00000000_00100000;
        const MakeDfpCenterExpandBypassCallback = 0b00000000_01000000;
    }
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct ModuleMapExternal0(u8);
bitflags! {
    impl ModuleMapExternal0: u8 {
        const UnderflowAndErrorReporting = 0b00000001;
        const CoprocBuild = 0b00000010;
    }
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct ClockPtrsToken {
    pub pll_info_table_ptr: u32,
    pub vbe_mode_pclk_table_ptr: u32,
    pub clocks_table_ptr: u32,
    pub clocks_programming_table_ptr: u32,
    pub nafll_table_ptr: u32,
    pub adc_table_ptr: u32,
    pub frequency_controller_table_ptr: u32,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct DfpPtrsToken {
    pub fp_established_ptr: u16,
    pub fp_table_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct NvinitPtrsToken {
    pub init_script_table_ptr: u16,
    pub macro_index_table_ptr: u16,
    pub macro_table_ptr: u16,
    pub condition_table_ptr: u16,
    pub io_condition_table_ptr: u16,
    pub io_flag_condition_table_ptr: u16,
    pub init_function_table_ptr: u16,
    pub vbios_private_boot_script_ptr: u16,
    pub data_arrays_table_ptr: u16,
    pub pcie_settings_script_ptr: u16,
    pub devinit_tables_ptr: u16,
    pub devinit_tables_size: u16,
    pub boot_scripts_ptr: u16,
    pub boot_scripts_size: u16,
    pub nvlink_configuration_data_ptr: u16,
    pub boot_scripts_non_gc6_ptr: u16,
    pub boot_scripts_size_non_gc6: u16,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct LvdsPtrsToken {
    pub lvds_info_table_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct MemoryPtrsToken {
    pub memory_strap_data_count: u8,
    pub memory_strap_translation_table_ptr: u16,
    pub memory_information_table_ptr: u16,
    pub reserved: u64,
    pub memory_partition_information_table: u32,
    pub memory_script_list_ptr: u32,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct PerfPtrsToken {
    pub performance_table_ptr: u32,
    pub memory_clock_table_ptr: u32,
    pub memory_tweak_table_ptr: u32,
    pub power_control_table_ptr: u32,
    pub thermal_control_table_ptr: u32,
    pub thermal_device_table_ptr: u32,
    pub thermal_coolers_table_ptr: u32,
    pub performance_settings_script_ptr: u32,
    pub continuous_virtual_binning_table_ptr: u32,
    pub ventura_table_ptr: u32,
    pub power_sensors_table_ptr: u32,
    pub power_policy_table_ptr: u32,
    pub p_state_clock_range_table_ptr: u32,
    pub voltage_frequency_table_ptr: u32,
    pub virtual_p_state_table_ptr: u32,
    pub power_topology_table_ptr: u32,
    pub power_leakage_table_ptr: u32,
    pub performance_test_specifications_table_ptr: u32,
    pub thermal_channel_table_ptr: u32,
    pub thermal_adjustment_table_ptr: u32,
    pub thermal_policy_table_ptr: u32,
    pub p_state_memory_clock_frequency_table_ptr: u32,
    pub fan_cooler_table_ptr: u32,
    pub fan_policy_table_ptr: u32,
    pub didt_table_ptr: u32,
    pub fan_test_table_ptr: u32,
    pub voltage_rail_table_ptr: u32,
    pub voltage_device_table_ptr: u32,
    pub voltage_policy_table_ptr: u32,
    pub low_power_table_ptr: u32,
    pub low_power_pcie_table_ptr: u32,
    pub low_power_pcie_platform_table_ptr: u32,
    pub low_power_gr_table_ptr: u32,
    pub low_power_ms_table_ptr: u32,
    pub low_power_di_table_ptr: u32,
    pub low_power_gc6_table_ptr: u32,
    pub low_power_psi_table_ptr: u32,
    pub thermal_monitor_table_ptr: u32,
    pub overclocking_table_ptr: u32,
    pub low_power_nvlink_table_ptr: u32,
}

#[derive(BinRead, Debug, Clone, Copy, Serialize)]
pub struct StringPtrsToken {
    pub sign_on_message_ptr: u16,
    pub sign_on_message_maximum_length: u8,
    pub version_string_ptr: u16,
    pub version_string_size: u8,
    pub copyright_string_ptr: u16,
    pub copyright_string_size: u8,
    pub oem_string_ptr: u16,
    pub oem_string_size: u8,
    pub oem_vendor_name_ptr: u16,
    pub oem_vendor_name_size: u8,
    pub oem_product_name_ptr: u16,
    pub oem_product_name_size: u8,
    pub oem_product_revision_ptr: u16,
    pub oem_product_revision_size: u8,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(ptrs: StringPtrsToken))]
pub struct StringToken {
    #[br(seek_before = SeekFrom::Start(ptrs.sign_on_message_ptr as u64))]
    #[br(count = ptrs.sign_on_message_maximum_length)]
    #[br(map = try_map_to_string)]
    #[br(if(ptrs.sign_on_message_ptr > 0))]
    pub sign_on_message: Option<String>,
    #[br(seek_before = SeekFrom::Start(ptrs.version_string_ptr as u64))]
    #[br(count = ptrs.version_string_size)]
    #[br(map = try_map_to_string)]
    #[br(if(ptrs.version_string_ptr > 0))]
    pub version_string: Option<String>,
    #[br(seek_before = SeekFrom::Start(ptrs.copyright_string_ptr as u64))]
    #[br(count = ptrs.copyright_string_size)]
    #[br(map = try_map_to_string)]
    #[br(if(ptrs.copyright_string_ptr > 0))]
    pub copyright_string: Option<String>,
    #[br(seek_before = SeekFrom::Start(ptrs.oem_string_ptr as u64))]
    #[br(count = ptrs.oem_string_size)]
    #[br(map = try_map_to_string)]
    #[br(if(ptrs.oem_string_ptr > 0))]
    pub oem_string: Option<String>,
    #[br(seek_before = SeekFrom::Start(ptrs.oem_vendor_name_ptr as u64))]
    #[br(count = ptrs.oem_vendor_name_size)]
    #[br(map = try_map_to_string)]
    #[br(if(ptrs.oem_vendor_name_ptr > 0))]
    pub oem_vendor_name: Option<String>,
    #[br(seek_before = SeekFrom::Start(ptrs.oem_product_name_ptr as u64))]
    #[br(count = ptrs.oem_product_name_size)]
    #[br(map = try_map_to_string)]
    #[br(if(ptrs.oem_product_name_ptr > 0))]
    pub oem_product_name: Option<String>,
    #[br(seek_before = SeekFrom::Start(ptrs.oem_product_revision_ptr as u64))]
    #[br(count = ptrs.oem_product_revision_size)]
    #[br(map = try_map_to_string)]
    #[br(if(ptrs.oem_product_revision_ptr > 0))]
    pub oem_product_revision: Option<String>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct TmdsPtrsToken {
    pub tmds_info_table_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct DisplayPtrsToken {
    pub display_scripting_table_ptr: u16,
    pub display_control_flags: DisplayControlFlags,
    pub sli_table_header_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct DisplayControlFlags(u8);
bitflags! {
    impl DisplayControlFlags: u8 {
        const EnableWhiteOverscanBorderForDiagnosticPurposes = 0b00000001;
        const NoDisplaySubsystem = 0b00000010;
        const DisplayFpga = 0b00000100;
        const VbiosAvoidsTouchingMempoolWhileDriversRunning = 0b00001000;
        const OffsetPclkBetween2Heads = 0b00010000;
        const BootWithDpHotplugDisabled = 0b00100000;
        const AllowDetectionOfDpSinksByDoingADpcdRegisterRead = 0b01000000;
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct VirtualPtrsToken {
    pub virtual_strap_field_table_ptr: u16,
    pub virtual_strap_field_register: u16,
    pub translation_table_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct DpPtrsToken {
    pub dp_info_table_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct DcbPtrsToken {
    pub dcb_header_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct FalconDataToken {
    pub falcon_ucode_table_ptr: u32,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct UefiDataToken {
    pub minimum_uefi_driver_version: u32,
    pub uefi_compatibility_level: u8,
    pub uefi_flags: UefiFlags,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct UefiFlags(u64);
bitflags! {
    impl UefiFlags: u64 {
        const DisplaySwitchSupport = 0b00000000_00000000_00000000_00000001;
        const LcdDiagnosticsSupport = 0b00000000_00000000_00000000_00000010;
        const GlitchlessSupport = 0b00000000_00000000_00000000_00000100;
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct MxmDataToken {
    pub module_spec_version: u8,
    pub module_flags: ModuleFlags,
    pub config_flags: ConfigFlags,
    pub dp_drive_strength_scale: u8,
    pub mxm_digital_connector_table_ptr: u16,
    pub mxm_aux_to_ccb_table_ptr: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct ModuleFlags(u8);
bitflags! {
    impl ModuleFlags: u8 {
        const NotMxm = 0x0;
        const TypeI = 0x1;
        const TypeII = 0x2;
        const TypeIII = 0x3;
        const TypeIV = 0x4;
        const Undefined = 0xF;
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct ConfigFlags(u8);
bitflags! {
    impl ConfigFlags: u8 {
        const NotMxm = 0b00000000;
        const MxmStructureRequired = 0b00000001;
        const MxmStructureValidationFailed = 0b00000010;
        const MxmDefaultDcb = 0b00001100;
        const OlderThanG3Package = 0b00000000;
        const G3Package = 0b00010000;
        const GB1_128_256Package = 0b00100000;
        const GB1_64Package = 0b00110000;
        const GB4_256Package = 0b01000000;

    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct BridgeFwDataToken {
    pub firmware_version: u32,
    pub firmware_oem_version: u8,
    pub firmware_image_length: u16,
    pub bios_mod_date: u64,
    pub firmware_flags: u32,
    pub engineering_product_name_ptr: u16,
    pub engineering_product_name_size: u8,
    pub instance_id: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(ptrs: ClockPtrsToken))]
pub struct PllInfo {
    #[br(seek_before = SeekFrom::Start(ptrs.pll_info_table_ptr as u64))]
    pub header: PllInfoHeader,
    #[br(count(header.entry_count))]
    pub entries: Vec<PllInfoEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct PllInfoHeader {
    pub version: u8,
    pub header_size: u8,
    #[br(assert(entry_size == 19))]
    pub entry_size: u8,
    pub entry_count: u8,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct PllInfoEntry {
    pub id: u8,
    pub ref_min_mhz: u16,
    pub ref_max_mhz: u16,
    pub vco_min_mhz: u16,
    pub vco_max_mhz: u16,
    pub update_min_mhz: u16,
    pub update_max_mhz: u16,
    pub m_min: u8,
    pub m_max: u8,
    pub n_min: u8,
    pub n_max: u8,
    pub pl_min: u8,
    pub pl_max: u8,
}

// #[derive(BinRead, Debug, Clone, Serialize)]
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
