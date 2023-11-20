// SPDX-License-Identifier: MIT

use crate::cursor::ContinuousRegionReader;
use crate::nvidia::bit::nvlink::NvLinkConfigData;
use crate::nvidia::bit::perf::{
    MemoryClockTable, MemoryTweakTable, PowerPolicyTable, VirtualPStateTable20,
};
use crate::nvidia::bit::{BITStructure, BITTokenType, PllInfo, StringToken};
use crate::nvidia::dcb::{
    CommunicationsControlBlock, ConnectorTable, DeviceControlBlock, GpioAssignmentTable,
    I2cDevicesTable,
};
use crate::nvidia::nbsi::NbsiPciExpansionRom;
use crate::nvidia::{NvgiRegion, NvidiaPciExpansionRom, RfrdRegion};
use crate::pci_efi::EfiPciExpansionRom;
use crate::pci_legacy::PciExpansionRom;
use crate::{FirmwareRegion, Region, RegionIterator, RegionStructure, RegionStructureIterator};
use binread::BinReaderExt;
use log::warn;
use serde::Serialize;
use std::io::{Read, Seek, SeekFrom};
use std::mem;

#[derive(Default, Debug, Serialize)]
pub struct FirmwareBundleInfo {
    pub firmwares: Vec<FirmwareInfo>,

    pub nbsi_pci_expansion_rom: Option<NbsiPciExpansionRom>,
}

#[derive(Default, Debug, Serialize)]
pub struct FirmwareInfo {
    pub nvgi_regions: Vec<NvgiRegion>,
    pub rfrd_region: Option<RfrdRegion>,
    pub legacy_pci_image: Option<LegacyPciImageInfo>,
    pub efi_pci_image: Option<EfiPciExpansionRom>,
    pub nv_pci_expansion_roms: Vec<NvidiaPciExpansionRom>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LegacyPciImageInfo {
    pub image: PciExpansionRom,

    // BIT
    pub bit_table_structure: Option<BITStructure>,
    pub bit_tokens_data: Vec<BITTokenType>,
    pub bit_string_token: Option<StringToken>,
    pub nvlink_config_data: Option<NvLinkConfigData>,
    pub memory_clock_table: Option<MemoryClockTable>,
    pub memory_tweak_table: Option<MemoryTweakTable>,
    pub pll_info: Option<PllInfo>,
    pub power_policy_table: Option<PowerPolicyTable>,
    pub virtual_p_state_table: Option<VirtualPStateTable20>,

    // DCB
    pub device_control_block: Option<DeviceControlBlock>,
    pub gpio_assignment_table: Option<GpioAssignmentTable>,
    pub i2c_devices_table: Option<I2cDevicesTable>,
    pub connector_table: Option<ConnectorTable>,
    pub communications_control_block: Option<CommunicationsControlBlock>,
}

impl FirmwareBundleInfo {
    pub fn parse<S: Read + Seek>(source: &mut S) -> crate::Result<Self> {
        let mut firmware_bundle = FirmwareBundleInfo::default();
        let mut firmware = FirmwareInfo::default();
        let mut firmwares: Vec<FirmwareInfo> = Vec::new();
        let mut region_iterator = RegionIterator::new(source);

        while let Some(region) = region_iterator.try_next()? {
            match region {
                Region::LegacyPciExpansionRom(legacy) => {
                    firmware.legacy_pci_image.replace(LegacyPciImageInfo {
                        image: legacy,
                        bit_table_structure: None,
                        bit_tokens_data: vec![],
                        bit_string_token: None,
                        nvlink_config_data: None,
                        memory_tweak_table: None,
                        memory_clock_table: None,
                        pll_info: None,
                        device_control_block: None,
                        gpio_assignment_table: None,
                        i2c_devices_table: None,
                        connector_table: None,
                        communications_control_block: None,
                        power_policy_table: None,
                        virtual_p_state_table: None,
                    });
                }
                Region::EfiPciExpansionRom(efi) => {
                    firmware.efi_pci_image.replace(efi);
                }
                Region::NvidiaPciExpansionRom(nv) => {
                    firmware.nv_pci_expansion_roms.push(nv);
                }
                Region::NbsiPciExpansionRom(nbsi) => {
                    firmware_bundle.nbsi_pci_expansion_rom.replace(nbsi);
                }
                Region::NvgiRegion(nvgi) => {
                    if firmware.rfrd_region.is_some() {
                        firmwares.push(mem::replace(&mut firmware, FirmwareInfo::default()));
                    }
                    firmware.nvgi_regions.push(nvgi);
                }
                Region::RfrdRegion(rfrd) => {
                    firmware.rfrd_region.replace(rfrd);
                }
            }
        }

        firmwares.push(mem::replace(&mut firmware, FirmwareInfo::default()));

        for firmware in &mut firmwares {
            Self::parse_legacy_pci_image_info(source, firmware)?;
        }
        firmware_bundle.firmwares = firmwares;
        Ok(firmware_bundle)
    }

    pub fn v_bios_info(&self) -> Vec<VBiosInfo> {
        self.firmwares
            .iter()
            .map(|f| {
                let mut info = VBiosInfo {
                    version: "N/A".to_string(),
                    gop_version: None,
                    subsystem_id: None,
                };

                if let Some(image) = &f.legacy_pci_image {
                    for bit_token in &image.bit_tokens_data {
                        if let BITTokenType::Bios(bios_token) = bit_token {
                            info.version = format!(
                                "{}.{:02X}",
                                bios_token.bios_version, bios_token.bios_oem_version
                            );
                        }
                    }
                    if let Some(ext) = &image.image.data_header_extended {
                        info.gop_version = ext
                            .gop_version
                            .map(|v| v.non_zero().map(|v| v.to_string()))
                            .flatten();
                        info.subsystem_id = ext
                            .subsystem_id
                            .map(|v| v.non_zero().map(|v| v.to_string()))
                            .flatten();
                    }
                }

                info
            })
            .collect()
    }

    fn parse_legacy_pci_image_info<S: Read + Seek>(
        source: &mut S,
        firmware: &mut FirmwareInfo,
    ) -> crate::Result<()> {
        if let Some(info) = firmware.legacy_pci_image.as_mut() {
            let mut legacy_image_regions: Vec<&dyn FirmwareRegion> = vec![&info.image];

            for nv in &firmware.nv_pci_expansion_roms {
                legacy_image_regions.push(nv);
            }
            let mut legacy_image_reader = ContinuousRegionReader::new(source, legacy_image_regions);
            legacy_image_reader.seek(SeekFrom::Start(info.image.header.pcir_offset as u64))?;
            let structures: Vec<RegionStructure> =
                RegionStructureIterator::new(&mut legacy_image_reader).collect();

            'structures_iteration: for structure in structures {
                match structure {
                    RegionStructure::BiosInformationTable(bit) => {
                        for token in &bit.tokens {
                            let bit_token_data = token.data(&mut legacy_image_reader);
                            match &bit_token_data {
                                Ok(BITTokenType::String(ptrs)) => {
                                    let string_token = legacy_image_reader
                                        .read_le_args::<StringToken>((ptrs.clone(),))?;
                                    info.bit_string_token.replace(string_token);
                                }
                                Ok(BITTokenType::NvInit(ptrs)) => {
                                    let nvlink_token = legacy_image_reader
                                        .read_le_args::<NvLinkConfigData>((ptrs.clone(),))?;
                                    info.nvlink_config_data.replace(nvlink_token);
                                }
                                Ok(BITTokenType::Clock(ptrs)) => {
                                    let pll_token = legacy_image_reader
                                        .read_le_args::<PllInfo>((ptrs.clone(),))?;
                                    info.pll_info.replace(pll_token);
                                }
                                Ok(BITTokenType::Perf(ptrs)) => {
                                    if ptrs.memory_clock_table_ptr > 0 {
                                        let memory_clock_table = legacy_image_reader
                                            .read_le_args::<MemoryClockTable>(
                                            (ptrs.clone(),),
                                        )?;
                                        info.memory_clock_table.replace(memory_clock_table);
                                    }

                                    if ptrs.memory_tweak_table_ptr > 0 {
                                        let memory_tweak_table = legacy_image_reader
                                            .read_le_args::<MemoryTweakTable>(
                                            (ptrs.clone(),),
                                        )?;
                                        info.memory_tweak_table.replace(memory_tweak_table);
                                    }

                                    if ptrs.virtual_p_state_table_ptr > 0 {
                                        let virtual_p_state_table = legacy_image_reader
                                            .read_le_args::<VirtualPStateTable20>(
                                            (ptrs.clone(),),
                                        )?;
                                        info.virtual_p_state_table.replace(virtual_p_state_table);
                                    }

                                    if ptrs.power_policy_table_ptr > 0 {
                                        let power_policy_table = legacy_image_reader
                                            .read_le_args::<PowerPolicyTable>(
                                            (ptrs.clone(),),
                                        )?;
                                        info.power_policy_table.replace(power_policy_table);
                                    }
                                }
                                Err(err) => {
                                    warn!("Failed to read token {:?}, error: {:?}", token, err);
                                }
                                _ => {}
                            }
                            if let Ok(bit_token_data) = bit_token_data {
                                info.bit_tokens_data.push(bit_token_data);
                            }
                        }

                        info.bit_table_structure.replace(bit);
                    }
                    RegionStructure::DeviceControlBlock(dcb) => {
                        if dcb.header.gpio_assignment_table_pointer > 0 {
                            legacy_image_reader.seek(SeekFrom::Start(
                                dcb.header.gpio_assignment_table_pointer as u64,
                            ))?;
                            let gpio_assignment_table =
                                legacy_image_reader.read_le::<GpioAssignmentTable>()?;
                            info.gpio_assignment_table.replace(gpio_assignment_table);
                        }

                        if dcb.header.i2c_devices_table_pointer > 0 {
                            legacy_image_reader.seek(SeekFrom::Start(
                                dcb.header.i2c_devices_table_pointer as u64,
                            ))?;
                            let i2c_devices_table =
                                legacy_image_reader.read_le::<I2cDevicesTable>()?;
                            info.i2c_devices_table.replace(i2c_devices_table);
                        }

                        if dcb.header.connector_table_pointer > 0 {
                            legacy_image_reader
                                .seek(SeekFrom::Start(dcb.header.connector_table_pointer as u64))?;
                            let connector_table =
                                legacy_image_reader.read_le::<ConnectorTable>()?;
                            info.connector_table.replace(connector_table);
                        }

                        if dcb.header.communications_control_block_pointer > 0 {
                            legacy_image_reader.seek(SeekFrom::Start(
                                dcb.header.communications_control_block_pointer as u64,
                            ))?;
                            let communications_control_block =
                                legacy_image_reader.read_le::<CommunicationsControlBlock>()?;
                            info.communications_control_block
                                .replace(communications_control_block);
                        }

                        info.device_control_block.replace(dcb);

                        break 'structures_iteration; // last parsed structure
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VBiosInfo {
    pub version: String,
    pub gop_version: Option<String>,
    pub subsystem_id: Option<String>,
}
