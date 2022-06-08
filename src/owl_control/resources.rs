use crate::owl_control::{crc32::Crc32, sysex::SysexData};
use anyhow::Result;
use byte_unit::Byte;
use wmidi::U7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resource {
    pub id: u8,
    pub name: String,
    pub size: u32,
    pub checksum: u32,
}

impl Resource {
    pub const fn new(id: u8, name: String, size: u32, checksum: u32) -> Self {
        Resource {
            id,
            name,
            size,
            checksum,
        }
    }
    pub fn size_string(&self) -> String {
        let byte = Byte::from_bytes(self.size.into());
        let adjusted_byte = byte.get_appropriate_unit(true);
        adjusted_byte.to_string()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResourceState {
    New,
    InProgress,
    Complete,
    Success,
    Failed,
    InvalidChecksum,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResourceData {
    pub state: ResourceState,
    pub data: Vec<u8>,
    pub crc: u32,
    pub current_idx: u32,
    pub offset: u32,
    pub size: u32,
    pub checksum: u32,
    decode_buffer: [u8; 256],
}

impl ResourceData {
    pub const fn new() -> Self {
        ResourceData {
            state: ResourceState::New,
            data: Vec::new(),
            crc: 0,
            current_idx: 0,
            offset: 0,
            size: 0,
            checksum: 0,
            decode_buffer: [0; 256],
        }
    }
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    pub fn process_data(&mut self, data: &[U7]) -> Result<()> {
        let old_offset = self.offset;
        match self.state {
            ResourceState::New => {
                if data.len() == 5 {
                    //let int_data = &data[..5];
                    if let Ok(_result) = self.size.decode(data) {
                        self.state = ResourceState::InProgress;
                        self.offset = 0;
                        self.crc = 0;
                    } else {
                        self.state = ResourceState::Failed;
                    }
                }
            }
            ResourceState::InProgress => {
                if let Ok(result) = self.decode_buffer.decode(data) {
                    self.offset += result.bytes_written as u32;
                    self.data
                        .extend(&self.decode_buffer[..result.bytes_written]);
                    //self.update_crc(data[5..size - 1]);
                    if self.offset >= self.size {
                        self.state = ResourceState::Complete;
                    }
                } else {
                    self.state = ResourceState::Failed;
                }
            }
            ResourceState::Complete => {
                if data.len() == 5 {
                    if let Ok(_result) = self.checksum.decode(data) {
                        self.state = ResourceState::New;
                        let crc = Crc32::new().update(&self.data[..self.size as usize]).crc;
                        if crc != self.checksum {
                            self.state = ResourceState::InvalidChecksum;
                        } else {
                            self.state = ResourceState::Success;
                        }
                    } else {
                        self.state = ResourceState::Failed;
                    }
                } else {
                    self.state = ResourceState::Failed;
                }
            }
            _ => {}
        }
        println!(
            "{} / {} (+{}, {:?})",
            self.offset,
            self.size,
            self.offset - old_offset,
            self.state
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_resource() {
        assert_eq!(
            Resource::new(0, "Test".to_string(), 100, 0).size_string(),
            "100 B"
        );
        assert_eq!(
            Resource::new(0, "Test".to_string(), 2048, 0).size_string(),
            "2.00 KiB"
        );
    }
}
