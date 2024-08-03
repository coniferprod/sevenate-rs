use std::convert::{
    From,
    TryFrom
};

use std::fmt;

use crate::dx7::{
    ParseError,
    MIDIChannel,
    Ranged
};

/// Parsing and generating MIDI System Exclusive data.
pub trait SystemExclusiveData: Sized {
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError>;
    fn to_bytes(&self) -> Vec<u8>;
    fn data_size(&self) -> usize;
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Format {
    Voice = 0,
    Cartridge = 9,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, "{}",
            match *self {
                Format::Voice => "voice",
                Format::Cartridge => "cartridge"
            })
    }
}

impl TryFrom<u8> for Format {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Format::Voice),
            9 => Ok(Format::Cartridge),
            _ => Err("Bad format value")
        }
    }
}

impl From<Format> for u8 {
    fn from(f: Format) -> u8 {
        f as u8
    }
}

pub struct Header {
    pub sub_status: u8,  // 0=voice/cartridge, 1=parameter
    pub channel: MIDIChannel,
    pub format: Format,
    pub byte_count: u16,  // 14-bit number distributed evenly over two bytes
    // voice=155 (00000010011011 = 0x009B, appears as "01 1B")
    // cartridge=4096 (01000000000000 = 0x1000, appears as "20 00")
}

impl fmt::Display for Header {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f, 
            "Format = {}, length = {} bytes", 
            self.format, self.byte_count)
    }
}

impl SystemExclusiveData for Header {
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        let byte_count_msb = data[2];
        let byte_count_lsb = data[3];
        let channel = ((data[0] & 0b00001111) + 1) as i32;
        if !MIDIChannel::is_valid(channel) {
            return Err(ParseError::InvalidData(1)) // offset of value
        }
        let format = Format::try_from(data[1]).expect("format should be valid");
        
        Ok(Self {
            sub_status: (data[0] >> 4) & 0b00000111,
            channel: MIDIChannel::new(channel),
            format,
            byte_count: match format {
                Format::Voice => 155,
                Format::Cartridge => 4069,
            }
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::new();

        let b0: u8 = self.channel.as_byte() | (self.sub_status << 4);
        result.push(b0);

        result.push(self.format.into());

        match self.format {
            Format::Voice => {
                result.push(0x01);
                result.push(0x1B);
            },
            Format::Cartridge => {
                result.push(0x20);
                result.push(0x00);
            }
        }

        result
    }

    fn data_size(&self) -> usize { 5 }
}

pub fn voice_checksum(data: &[u8]) -> u8 {
    let mut sum: u32 = 0;
    for b in data {
        sum += *b as u32;
    }

    let mut checksum: u8 = (sum & 0xff) as u8;
    checksum = !checksum;
    checksum &= 0x7f;
    checksum += 1;
    checksum
}
