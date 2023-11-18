// SPDX-License-Identifier: MIT

use binread::BinRead;
use bitflags::bitflags;
use modular_bitfield::prelude::*;
use serde::Serialize;
use std::fmt::Debug;

pub const DCB_SIGNATURE: &[u8] = b"\xcb\xbd\xdc\x4e";

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct DeviceControlBlock {
    #[br(parse_with = crate::stream_position)]
    pub offset_in_region: u64,
    #[br(restore_position)]
    pub header: DeviceControlBlockHeader,
    #[br(pad_before(header.header_size as i64))]
    pub unknown: u8,
    #[br(count(header.entry_count))]
    pub entries: Vec<DeviceEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct DeviceControlBlockHeader {
    #[br(parse_with = crate::stream_position)]
    pub offset_in_region: u64,
    pub version: u8,
    pub header_size: u8,
    pub entry_count: u8,
    pub entry_size: u8,
    pub communications_control_block_pointer: u16,
    #[br(assert(signature == DCB_SIGNATURE))]
    pub signature: [u8; 4],
    pub gpio_assignment_table_pointer: u16,
    pub input_devices_table_pointer: u16,
    pub personal_cinema_table_pointer: u16,
    pub spread_spectrum_table_pointer: u16,
    pub i2c_devices_table_pointer: u16,
    pub connector_table_pointer: u16,
    pub flags: DeviceControlBlockFlags,
    pub hdtv_translation_table_pointer: u16,
    pub switched_outputs_table_pointer: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct DeviceControlBlockFlags(u8);
bitflags! {
    impl DeviceControlBlockFlags: u8 {
        const BootDisplayCount1Allowed = 0b00000000;
        const BootDisplayCount2Allowed = 0b00000001;
        const NoVip = 0b00000000;
        const VipOnPinSetA = 0b00010000;
        const VipOnPinSetB = 0b00100000;
        const PinSetANotAttached = 0b00000000;
        const PinSetARoutedToSliFinger = 0b01000000;
        const PinSetBNotAttached = 0b00000000;
        const PinSetBRoutedToSliFinger = 0b10000000;
    }
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct DeviceEntry {
    #[br(restore_position)]
    #[br(pad_before(4))]
    pub display_path_information: DisplayPathInformation,

    #[br(args(display_path_information.display_type()))]
    #[br(pad_after(4))]
    pub device_specific_information: DeviceSpecificInformation,
}

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
#[br(map = |value: u32| Self::from_bytes(value.to_be_bytes()))]
pub struct DisplayPathInformation {
    pub display_type: DisplayType,
    pub edid_port: B4,

    pub head: B4,
    pub connector: B4,

    pub bus: B4,
    pub location: Location,
    pub is_boot_device_removed: bool,
    pub is_blind_boot_device_removed: bool,

    pub output_devices: B4,
    pub is_virtual_device: bool,
    pub reserved: B3,
}

#[derive(Debug, Copy, Clone, PartialEq, BitfieldSpecifier, Serialize)]
#[bits = 4]
pub enum DisplayType {
    Crt = 0x0,
    Tv = 0x1,
    Tmds = 0x2,
    Lvds = 0x3,
    Sdi = 0x5,
    DisplayPort = 0x6,
    EndOfLine = 0xE,
    SkipEntry = 0xF,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 2]
pub enum Location {
    OnChip = 0x0,
    OnBoard = 0x1,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(display_type: DisplayType))]
pub enum DeviceSpecificInformation {
    #[br(pre_assert(display_type == DisplayType::Crt))]
    Crt(u32),
    #[br(pre_assert(display_type == DisplayType::Tmds || display_type == DisplayType::Lvds || display_type == DisplayType::Sdi || display_type == DisplayType::DisplayPort))]
    Dfp(DfpDeviceSpecificInformation),
    #[br(pre_assert(display_type == DisplayType::Tv))]
    Tv(TvDeviceSpecificInformation),
    Extra(u32),
}

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
#[br(map = |value: u32| Self::from_bytes(value.to_be_bytes()))]
pub struct DfpDeviceSpecificInformation {
    pub edid_source: EdidSource,
    pub power_and_backlight_control: PowerAndBacklightControl,
    pub sub_link_b_dp_b_pad_link_1: bool,
    pub sub_link_a_dp_a_pad_link_0: bool,
    pub reserved_4: B2,

    pub external_link_type: ExternalLinkType,

    pub reserved_3: B1,
    pub hdmi_enable: bool,
    pub reserved_2: B2,
    pub external_communication_port: ExternalCommunicationsPort,
    pub maximum_link_rate: MaximumLinkRate,

    pub maximum_lane_count: MaximumLaneCount,
    pub reserved_1: B4,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 2]
pub enum EdidSource {
    Ddc = 0x0,
    PanelStrapsAndVBiosTables = 0x1,
    DdcAcpiOrBiosCalls = 0x2,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 8]
pub enum ExternalLinkType {
    UndefinedSingleLink = 0x0,
    SiliconImage164SingleLinkTmds = 0x1,
    SiliconImage178SingleLinkTmds = 0x2,
    DualSiliconImage178DualLinkTmds = 0x3,
    Chrontel7009SingleLinkTmds = 0x4,
    Chrontel7019DualLinkLvds = 0x5,
    NationalSemiconductorDs90C387DualLinkLvds = 0x6,
    SiliconImage164SingleLinkTmdsAlternateAddress = 0x7,
    Chrontel7301SingleLinkTmds = 0x8,
    SiliconImage1162SingleLinkTmdsAlternateAddress = 0x9,
    AnalogixAnx9801FourLaneDisplayPort = 0xB,
    ParadeTechDp5014LaneDisplayPort = 0xC,
    AnalogixAnx9805HdmiAndDisplayPort = 0xD,
    AnalogixAnx9805HdmiAndDisplayPortAlternateAddress = 0xE,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 1]
pub enum ExternalCommunicationsPort {
    Primary = 0x0,
    Secondary = 0x1,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 2]
pub enum PowerAndBacklightControl {
    External = 0x0,
    Scripts = 0x1,
    VBiosCallbacksToSBios = 0x2,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 3]
pub enum MaximumLinkRate {
    Rate1620Mbps = 0x0,
    Rate2700Mbps = 0x1,
    Rate5400Mbps = 0x2,
    Rate8100Mbps = 0x3,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 4]
pub enum MaximumLaneCount {
    SingleLine = 0x1,
    TwoLines = 0x2,
    TwoLinesDeprecated = 0x3,
    FourLines = 0x4,
    FourLinesDeprecated = 0xF,
}

fn map_tv_device_specification_information(value: u32) -> TvDeviceSpecificInformation {
    let bytes = value.to_be_bytes();
    let dacs: u8 = bytes[0] & 0x0F + bytes[2] & 0xF0;
    // [sdtv:3, rsvd:1, e:1, cc: 2, hdtv: 4, rsvd: 5, dacs: 8, encoder: 8]
    let bytes = [bytes[0] & 0xF0 + bytes[2] & 0x0F, bytes[3], dacs, bytes[1]];
    TvDeviceSpecificInformation::from_bytes(bytes)
}

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
#[br(map = map_tv_device_specification_information)]
pub struct TvDeviceSpecificInformation {
    pub sdtv_format: SdtvFormat,
    pub reserved_1: B1,
    pub external_communication_port: ExternalCommunicationsPort,
    pub connection_count: ConnectorCount,
    pub hdtv_format: HdtvFormat,
    pub reserved_2: B5,

    pub dacs: Dacs,

    pub encoder_identifier: EncoderIdentifier,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 3]
pub enum SdtvFormat {
    NtscM,
    NtscJ,
    PalM,
    PalBdghi,
    PalN,
    PalNC,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 8]
pub enum Dacs {
    CvbsOnGreen = 0x02,
    CvbsOnGreenSVideoOnRedAndGreen = 0x03,
    CvbsOnBlue = 0x04,
    CvbsOnBlueSVideoOnRedAndGreen = 0x07,
    StandardHdtv = 0x08,
    HdtvTwist1 = 0x09,
    Scart = 0x0A,
    Twist2 = 0x0B,
    ScartAndHdtv = 0x0C,
    StandardHdtvWithoutSdtv = 0x0D,
    ScartTwist1 = 0x0E,
    ScartAndHdtv_ = 0x0F,
    CompositeAndHdtv = 0x11,
    HdtvAndScartTwist1 = 0x12,
    SVideoOnRedAndGreen = 0x13,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 8]
pub enum EncoderIdentifier {
    Brooktree868 = 0x00,
    Brooktree869 = 0x01,
    Conexant870 = 0x02,
    Conexant871 = 0x03,
    Conexant872 = 0x04,
    Conexant873 = 0x05,
    Conexant874 = 0x06,
    Conexant875 = 0x07,

    Chrontel7003 = 0x40,
    Chrontel7004 = 0x41,
    Chrontel7005 = 0x42,
    Chrontel7006 = 0x43,
    Chrontel7007 = 0x44,
    Chrontel7008 = 0x45,
    Chrontel7009 = 0x46,
    Chrontel7010 = 0x47,
    Chrontel7011 = 0x48,
    Chrontel7012 = 0x49,
    Chrontel7019 = 0x4A,
    Chrontel7021 = 0x4B,

    Philips7102 = 0x80,
    Philips7103 = 0x81,
    Philips7104 = 0x82,
    Philips7105 = 0x83,
    Philips7108 = 0x84,
    Philips7108A = 0x85,
    Philips7108B = 0x86,
    Philips7109 = 0x87,
    Philips7109A = 0x88,

    NvidiaInternal = 0x0C,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 2]
pub enum ConnectorCount {
    SingleConnector,
    TwoConnectors,
    ThreeConnectors,
    FourConnectors,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 4]
pub enum HdtvFormat {
    Hdtv576I,
    Hdtv480I,
    Hdtv576P50Hz,
    Hdtv720P50Hz,
    Hdtv720P60Hz,
    Hdtv1080I50Hz,
    Hdtv1080I60Hz,
    Hdtv1080P24Hz,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct GpioAssignmentTable {
    pub header: GpioAssignmentTableHeader,
    #[br(count(header.entry_count))]
    #[br(args(header.entry_size))]
    pub entries: Vec<GpioAssignmentTableEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct GpioAssignmentTableHeader {
    pub version: u8,
    #[br(assert(header_size >= 6))]
    pub header_size: u8,
    pub entry_count: u8,
    #[br(assert(entry_size >= 5))]
    pub entry_size: u8,
    #[br(pad_after = header_size as i64 - 6)]
    pub ext_gpio_master: u16,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(import(entry_size: u8))]
pub struct GpioAssignmentTableEntry {
    pub pin: GpioEntryPin,
    #[br(restore_position)]
    #[br(try)]
    pub function: Option<GpioEntryFunction>,
    pub function_raw: u8,
    pub output: u8,
    pub input: GpioEntryInput,
    #[br(pad_after = entry_size as i64 - 5)]
    pub misc: GpioEntryMisc,
}

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
pub struct GpioEntryPin {
    pub pin_number: B6,
    pub io_type: bool,
    pub init_state: bool,
}

// More: https://nvidia.github.io/open-gpu-doc/DCB/DCB-4.x-Specification.html
#[derive(BinRead, Debug, Clone, Serialize)]
#[repr(u8)]
#[br(repr = u8)]
pub enum GpioEntryFunction {
    HotPlugA = 7,
    HotPlugB = 8,
    FanControl = 9,
    ThermalEvent = 17,
    OverTemp = 35,
    GenericInitialized = 48,
    ThermalAlert = 52,
    ThermalCritical = 53,
    FanSpeedSense = 61,
    PowerAlert = 76,
    HotPlugC = 81,
    HotPlugD = 82,
    HotPlugE = 94,
    HotPlugF = 95,
    HotPlugG = 96,
    NvddPsi = 122,
    NvvddPwm = 129,
    InstanceId0 = 209,
    InstanceId1 = 210,
    InstanceId2 = 211,
    InstanceId3 = 212,
    InstanceId4 = 213,
    InstanceId5 = 214,
    InstanceId6 = 215,
    InstanceId7 = 216,
    InstanceId8 = 217,
    InstanceId9 = 218,

    SkipEntry = 0xFF,
}

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
pub struct GpioEntryInput {
    pub hw_select: GpioEntryInputHwSelect,
    pub g_sync: bool,
    pub open_drain: bool,
    pub pwm: bool,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 5]
pub enum GpioEntryInputHwSelect {
    None = 0,
    ThermalAlert = 22,
    PowerAlert = 23,
}

#[bitfield]
#[derive(Copy, Clone, Debug, BinRead, Serialize)]
pub struct GpioEntryMisc {
    pub lock: B4,
    pub io: GpioEntryMiscIo,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 4]
pub enum GpioEntryMiscIo {
    Unused = 0x0,
    InvOut = 0x1,
    InvOutTristate = 0x3,
    Out = 0x4,
    InStereoTristate = 0x6,
    InvOutTristateLo = 0x9,
    InvIn = 0xB,
    OutTristate = 0xC,
    IoIn = 0xE,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct I2cDevicesTable {
    pub header: I2cDevicesTableHeader,
    #[br(count(header.entry_count))]
    pub entries: Vec<I2cDevicesTableEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct I2cDevicesTableHeader {
    pub version: u8,
    #[br(assert(header_size >= 5))]
    pub header_size: u8,
    pub entry_count: u8,
    #[br(assert(entry_size == 4))]
    pub entry_size: u8,
    #[br(pad_after = header_size as i64 - 5)]
    pub flags: I2cDevicesTableHeaderFlags,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize)]
//#[br(map = |value: u32| Self::from_bytes(value.to_be_bytes()))]
pub struct I2cDevicesTableEntry {
    pub device_type: I2cDevicesTableEntryDeviceType,
    pub i2c_address: u8,
    pub reserved_0: B4,
    pub external_communications_port: B1,
    pub write_access_privilege_level: B3,
    pub read_access_privilege_level: B3,
    pub reserved_1: B5,
}

#[derive(Debug, Clone, BitfieldSpecifier, Serialize)]
#[bits = 8]
pub enum I2cDevicesTableEntryDeviceType {
    // Thermal Chips
    Adm1032 = 0x01,
    Max6649 = 0x02,
    Lm99 = 0x03,
    Max1617 = 0x06,
    Lm64 = 0x07,
    Adt7473 = 0x0A,
    Lm89 = 0x0B,
    Tmp411 = 0x0C,
    Adt7461 = 0x0D,
    // I2C ADC
    Ads1112 = 0x30,
    // I2c Power Controllers
    Pic16F690 = 0xC0,
    Vt1103 = 0x40,
    Px3540 = 0x41,
    Vt1165 = 0x42,
    ChiL8203_8212_8213_8214 = 0x43,
    Ncp4208 = 0x44,

    SkipEntry = 0xFF,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct I2cDevicesTableHeaderFlags(u8);
bitflags! {
    impl I2cDevicesTableHeaderFlags: u8 {
        const DisableDeviceProbing = 0b10000000;
    }
}

// https://nvidia.github.io/open-gpu-doc/DCB/DCB-4.x-Specification.html#_connector_table
#[derive(BinRead, Debug, Clone, Serialize)]
pub struct ConnectorTable {
    pub header: ConnectorTableHeader,
    #[br(count(header.entry_count))]
    pub entries: Vec<ConnectorTableEntry>,
}

#[derive(BinRead, Debug, Clone, Serialize)]
pub struct ConnectorTableHeader {
    pub version: u8,
    #[br(assert(header_size >= 5))]
    pub header_size: u8,
    pub entry_count: u8,
    #[br(assert(entry_size == 4))]
    pub entry_size: u8,
    pub platform: ConnectorTablePlatform,
}

#[bitfield]
#[derive(BinRead, Debug, Clone, Serialize)]
pub struct ConnectorTableEntry {
    pub connector_type: ConnectorType,

    pub location: B4,
    pub hotplug_a_interrupt: bool,
    pub hotplug_b_interrupt: bool,
    pub dp_a: bool,
    pub dp_b: bool,

    pub hotplug_c_interrupt: bool,
    pub hotplug_d_interrupt: bool,
    pub dp_c: bool,
    pub dp_d: bool,
    pub di_a: bool,
    pub di_b: bool,
    pub di_c: bool,
    pub di_d: bool,

    pub hotplug_e_interrupt: bool,
    pub hotplug_f_interrupt: bool,
    pub hotplug_g_interrupt: bool,
    pub self_refresh_a: bool,
    pub lcd_interrupt_gpio_pin: B3,
    pub reserved: B1,
}

#[derive(BinRead, Debug, Clone, Serialize)]
#[br(repr = u8)]
#[repr(u8)]
pub enum ConnectorTablePlatform {
    NormalAddInCard = 0x00,
    TwoBackPlateAddInCards = 0x01,
    AddInCardConfigurable = 0x02,
    DesktopWithIntegratedFullDp = 0x07,
    MobileAddInCard = 0x08,
    MxmModule = 0x09,
    MobileSystemWithAllDisplaysOnTheBackOfTheSystem = 0x10,
    MobileSystemWithDisplayConnectorsOnTheBackAndLeftOfTheSystem = 0x11,
    MobileSystemWithExtraConnectorsOnTheDock = 0x18,
    CrushNormalBackPlateDesign = 0x20,
}

#[derive(BinRead, Debug, Clone, BitfieldSpecifier, Serialize)]
#[br(repr = u8)]
#[repr(u8)]
#[bits = 8]
pub enum ConnectorType {
    Vga15Pin = 0x00,
    DviA = 0x01,
    PodVga15Pin = 0x02,
    TvCompositeOut = 0x10,
    TvSVideoOut = 0x11,
    TvSVideoBreakoutComposite = 0x12,
    TvHdtvComponentYPrPb = 0x13,
    TvScart = 0x14,
    TvCompositeScartOverBlue = 0x16,
    TvHdtvEiaj4120 = 0x17,
    PodHdtvYPrPb = 0x18,
    PodSVideo = 0x19,
    PodComposite = 0x1A,
    DviITvSVideo = 0x20,
    DviITvComposite = 0x21,
    DviITvSVideoBreakoutComposite = 0x22,
    DviI = 0x30,
    DviD = 0x31,
    AppleDisplayConnector = 0x32,
    LfhDviI1 = 0x38,
    LfhDviI2 = 0x39,
    Bnc = 0x3C,
    LvdsSpwgAttached = 0x40,
    LvdsOemAttached = 0x41,
    LvdsSpwgDetached = 0x42,
    LvdsOemDetached = 0x43,
    TmdsOemAttached = 0x45,
    DisplayPortExternalConnector = 0x46,
    DisplayPortInternalConnector = 0x47,
    DisplayPortMiniExternalConnector = 0x48,
    Vga15PinIfNotDocked = 0x50,
    Vga15PinIfDocked = 0x51,
    DviIIfNotDocked = 0x52,
    DviIIfDocked = 0x53,
    DviDIfNotDocked = 0x54,
    DviDIfDocked = 0x55,
    DisplayPortExternalIfNotDocked = 0x56,
    DisplayPortExternalIfDocked = 0x57,
    DisplayPortMiniExternalIfNotDocked = 0x58,
    DisplayPortMiniExternalIfDocked = 0x59,
    ThreePinDinStereoConnector = 0x60,
    HdmiAConnector = 0x61,
    AudioSpdifConnector = 0x62,
    HdmiCMiniConnector = 0x63,
    LfhDp1 = 0x64,
    LfhDp2 = 0x65,
    VirtualConnectorForWifiDisplay = 0x70,

    SkipEntry = 0xFF,
}
