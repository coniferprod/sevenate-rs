use std::convert::{
    From,
    TryFrom
};

use std::fmt;
use syxpack::{
    Ranged,
    ParseError,
    MidiChannel,
    Encoding,
    SystemExclusiveData,
};

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
    pub channel: MidiChannel,
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
    fn parse(data: &[u8]) -> Result<Self, ParseError> {
        //let byte_count_msb = data[2];
        //let byte_count_lsb = data[3];
        let channel = ((data[0] & 0b00001111) + 1) as i32;
        if !MidiChannel::contains(channel) {
            return Err(ParseError::InvalidData(
                1, // offset of value
                format!("Invalid MIDI channel value (raw: {:02X})", data[0])));
        }
        let format = Format::try_from(data[1]).expect("format should be valid");

        Ok(Self {
            sub_status: (data[0] >> 4) & 0b00000111,
            channel: MidiChannel::new(channel),
            format,
            byte_count: match format {
                Format::Voice => crate::dx7::voice::VOICE_SIZE as u16,
                Format::Cartridge => crate::dx7::cartridge::CARTRIDGE_DATA_SIZE as u16,
            }
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::new();

        let b0: u8 = self.channel.encode() | (self.sub_status << 4);
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

    fn data_size() -> usize { 4 }
}

pub fn checksum(data: &[u8]) -> u8 {
    let sum: u32 = data.iter().fold(0, |a, &b| a.wrapping_add(b as u32));
    let mut checksum = sum & 0xff;
    checksum = !checksum + 1;
    checksum &= 0x7f;
    checksum as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        // Yamaha DX7 original ROM1A sound bank (data only, no SysEx header/terminator
        // or checksum.)
        let rom1a_data = include_bytes!("rom1a_payload.dat");
        let first_index = 4;
        let last_index = rom1a_data.len() - 2;

        let data = &rom1a_data[first_index..=last_index];
        assert_eq!(data.len(), crate::dx7::cartridge::CARTRIDGE_DATA_SIZE);

        // The original checksum is 0x33
        let cs = checksum(data);
        assert_eq!(0x33, cs);
    }
}
