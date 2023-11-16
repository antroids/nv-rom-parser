// SPDX-License-Identifier: MIT

use crate::FirmwareRegion;
use log::trace;
use std::io::{Read, Seek, SeekFrom};
use std::ops::{AddAssign, SubAssign};

pub struct ContinuousRegionReader<'a, S> {
    pub source: &'a mut S,
    pub translated_stream_position: u64,

    pub regions: Vec<&'a dyn FirmwareRegion>,
}

impl<'a, S> ContinuousRegionReader<'a, S> {
    pub fn new(source: &'a mut S, mut regions: Vec<&'a dyn FirmwareRegion>) -> Self {
        regions.sort_by_key(|r| r.offset_in_firmware());
        Self {
            source,
            translated_stream_position: 0,
            regions,
        }
    }

    fn reader_position_info(&self, firmware_position: u64) -> ReaderPositionInfo<'a> {
        let mut current_region_translated_offset = 0u64;
        let mut end_offset_in_firmware = 0u64;
        for (region_index, region) in self.regions.iter().enumerate() {
            let offset_in_firmware = region.offset_in_firmware();
            let region_size = region.region_size();
            end_offset_in_firmware = offset_in_firmware + region_size;
            if end_offset_in_firmware <= firmware_position {
                current_region_translated_offset.add_assign(region_size);
            } else if offset_in_firmware <= firmware_position {
                let offset = firmware_position - offset_in_firmware;
                return ReaderPositionInfo::InRegion {
                    region: *region,
                    region_index,
                    offset,
                    translated_position: current_region_translated_offset + offset,
                };
            } else if current_region_translated_offset == 0 {
                return ReaderPositionInfo::BeforeFirstRegion;
            } else {
                return ReaderPositionInfo::BetweenRegions;
            }
        }

        ReaderPositionInfo::AfterLastRegion {
            translated_position: firmware_position - end_offset_in_firmware
                + current_region_translated_offset,
        }
    }
}

#[derive(Debug)]
enum ReaderPositionInfo<'a> {
    BeforeFirstRegion,
    InRegion {
        region: &'a dyn FirmwareRegion,
        region_index: usize,
        offset: u64,
        translated_position: u64,
    },
    BetweenRegions,
    AfterLastRegion {
        translated_position: u64,
    },
}

impl<'a, S: Read + Seek> Read for ContinuousRegionReader<'a, S> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let firmware_position = self.source.stream_position()?;
        let position_info = self.reader_position_info(firmware_position);

        match position_info {
            ReaderPositionInfo::InRegion {
                region,
                region_index,
                offset,
                ..
            } => {
                let bytes_left_to_read = region.region_size() - offset;
                let buf_len = buf.len().min(bytes_left_to_read as usize);
                let read_count = self.source.read(&mut buf[..buf_len])?;
                if read_count == bytes_left_to_read as usize {
                    if let Some(next_region) = self.regions.get(region_index + 1) {
                        self.source
                            .seek(SeekFrom::Start(next_region.offset_in_firmware()))?;
                    }
                }
                Ok(read_count)
            }
            ReaderPositionInfo::AfterLastRegion { .. } => Ok(0),
            ReaderPositionInfo::BeforeFirstRegion => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot read before first region!",
            )),
            ReaderPositionInfo::BetweenRegions => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Cannot read between specified regions!",
            )),
        }
    }
}

impl<'a, S: Read + Seek> Seek for ContinuousRegionReader<'a, S> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        return match pos {
            SeekFrom::Start(from_start) => {
                let mut remaining_offset = from_start;
                for region in &self.regions {
                    let region_size = region.region_size();
                    if region_size > remaining_offset {
                        let seek_in_firmware = region.offset_in_firmware() + remaining_offset;
                        trace!(
                            "Seek translated {}, in firmware {}",
                            from_start,
                            seek_in_firmware
                        );
                        self.source.seek(SeekFrom::Start(seek_in_firmware))?;
                        return Ok(from_start);
                    } else {
                        remaining_offset.sub_assign(region_size)
                    }
                }
                let last_region_end_offset = self
                    .regions
                    .last()
                    .map(|r| r.end_offset_in_firmware())
                    .unwrap_or(0);
                self.source
                    .seek(SeekFrom::Start(last_region_end_offset + remaining_offset))
            }
            SeekFrom::End(from_end) => {
                let total_regions_size: u64 = self.regions.iter().map(|r| r.region_size()).sum();
                if let Some(translated_seek_position) =
                    total_regions_size.checked_add_signed(from_end)
                {
                    self.seek(SeekFrom::Start(translated_seek_position))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid seek to a negative or overflowing position!",
                    ))
                }
            }
            SeekFrom::Current(from_current) => {
                let firmware_position = self.source.stream_position()?;
                let position_info = self.reader_position_info(firmware_position);
                let translated_position = match position_info {
                    ReaderPositionInfo::InRegion {
                        translated_position,
                        ..
                    } => translated_position,
                    ReaderPositionInfo::AfterLastRegion {
                        translated_position,
                        ..
                    } => translated_position,
                    ReaderPositionInfo::BetweenRegions | ReaderPositionInfo::BeforeFirstRegion => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Cannot relative seek from outside of specified regions!",
                        ))
                    }
                };

                if let Some(translated_seek_position) =
                    translated_position.checked_add_signed(from_current)
                {
                    self.seek(SeekFrom::Start(translated_seek_position))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid seek to a negative or overflowing position!",
                    ))
                }
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::cursor::ContinuousRegionReader;
    use crate::FirmwareRegion;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    #[derive(Debug)]
    struct TestRegion {
        start: u64,
        size: u64,
    }

    impl FirmwareRegion for TestRegion {
        fn offset_in_firmware(&self) -> u64 {
            self.start
        }

        fn region_size(&self) -> u64 {
            self.size
        }
    }

    #[test]
    fn test_read() {
        let data = Vec::from_iter(0u8..100);
        let region_1 = TestRegion { start: 0, size: 10 };
        let _region_2 = TestRegion { start: 10, size: 5 };
        let region_3 = TestRegion {
            start: 15,
            size: 35,
        };
        let _region_4 = TestRegion {
            start: 50,
            size: 30,
        };
        let region_5 = TestRegion {
            start: 80,
            size: 10,
        };
        let _region_6 = TestRegion {
            start: 90,
            size: 10,
        };

        let mut cursor = Cursor::new(data.as_slice());
        // [0..10; 15..50; 80..90]
        let mut reader =
            ContinuousRegionReader::new(&mut cursor, vec![&region_1, &region_3, &region_5]);
        let mut buf = [0u8; 10];

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(data[0..10], buf);

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(data[15..25], buf);

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(data[25..35], buf);

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(data[35..45], buf);

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(data[45..50], buf[..5]);
        assert_eq!(data[80..85], buf[5..]);

        reader.read_exact(&mut buf[..5]).unwrap();
        assert_eq!(data[85..90], buf[..5]);

        assert!(reader.read_exact(&mut buf[..5]).is_err());
    }

    #[test]
    fn test_seek() {
        let data = Vec::from_iter(0u8..100);
        let region_1 = TestRegion { start: 0, size: 10 };
        let _region_2 = TestRegion { start: 10, size: 5 };
        let region_3 = TestRegion {
            start: 15,
            size: 35,
        };
        let _region_4 = TestRegion {
            start: 50,
            size: 30,
        };
        let region_5 = TestRegion {
            start: 80,
            size: 10,
        };
        let _region_6 = TestRegion {
            start: 90,
            size: 10,
        };

        let mut cursor = Cursor::new(data.as_slice());
        // [0..10; 15..50; 80..90]
        let mut reader =
            ContinuousRegionReader::new(&mut cursor, vec![&region_1, &region_3, &region_5]);
        let mut buf = [0u8; 1];

        reader.read_exact(&mut buf).unwrap();
        assert_eq!(0, buf[0]);

        assert_eq!(5, reader.seek(SeekFrom::Start(5)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(5, buf[0]);

        assert_eq!(8, reader.seek(SeekFrom::Current(2)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(8, buf[0]);

        assert_eq!(19, reader.seek(SeekFrom::Current(10)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(24, buf[0]);

        assert_eq!(30, reader.seek(SeekFrom::Current(10)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(35, buf[0]);

        assert_eq!(11, reader.seek(SeekFrom::Current(-20)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(16, buf[0]);

        assert!(reader.seek(SeekFrom::Current(-20)).is_err());

        assert_eq!(54, reader.seek(SeekFrom::Start(54)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(89, buf[0]);

        assert!(reader.read_exact(&mut buf).is_err());

        assert_eq!(54, reader.seek(SeekFrom::End(-1)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(89, buf[0]);

        assert!(reader.read_exact(&mut buf).is_err());

        assert_eq!(35, reader.seek(SeekFrom::Current(-20)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(40, buf[0]);

        assert_eq!(0, reader.seek(SeekFrom::Current(-36)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(0, buf[0]);

        assert_eq!(0, reader.seek(SeekFrom::End(-55)).unwrap());
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(0, buf[0]);
    }
}
