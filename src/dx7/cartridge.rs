use log::debug;
use dbg_hex::dbg_hex;

use crate::ParseError;
use crate::dx7::voice::{
    Voice,
    VOICE_PACKED_SIZE
};
use crate::dx7::sysex::SystemExclusiveData;

pub const VOICE_COUNT: usize = 32;
pub const CARTRIDGE_DATA_SIZE: usize = 4096;

/// A DX7 cartridge with 32 voices.
#[derive(Debug)]
pub struct Cartridge {
    pub voices: Vec<Voice>,
}

impl Default for Cartridge {
    fn default() -> Self {
        Cartridge {
            voices: vec![Default::default(); VOICE_COUNT],
        }
    }
}

impl SystemExclusiveData for Cartridge {
    fn parse(data: &[u8]) -> Result<Self, ParseError> {
        let mut offset = 0;
        let mut voices = Vec::<Voice>::new();
        for _i in 0..VOICE_COUNT {
            //eprintln!("VOICE {}", i + 1);
            let packed_voice_data = &data[offset..offset + VOICE_PACKED_SIZE];
            //eprintln!("Packed voice data length = {}", packed_voice_data.len());
            //dbg_hex!(&packed_voice_data);
            let voice_data = Voice::unpack(packed_voice_data);
            //eprintln!("Unpacked voice data length = {}", voice_data.len());
            //dbg_hex!(&voice_data);
            let voice = Voice::parse(&voice_data).expect("valid voice data");
            //eprintln!("name = {}", voice.name);
            voices.push(voice);
            offset += VOICE_PACKED_SIZE;
        }
        Ok(Cartridge { voices })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        for (index, voice) in self.voices.iter().enumerate() {
            let voice_data = voice.to_bytes();
            let packed_voice_data = Voice::pack(&voice_data);
            debug!("Voice #{} packed data length = {} bytes", index, voice_data.len());
            data.extend(packed_voice_data);
        }

        data
    }

    const DATA_SIZE: usize = CARTRIDGE_DATA_SIZE;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cartridge_length() {
        let cartridge = Cartridge::default();
        assert_eq!(cartridge.to_bytes().len(), CARTRIDGE_DATA_SIZE);
    }
}
