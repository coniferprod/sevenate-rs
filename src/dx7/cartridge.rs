use log::debug;

use crate::ParseError;
use crate::dx7::voice::Voice;
use crate::dx7::sysex::SystemExclusiveData;

const VOICE_COUNT: usize = 32;

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
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        let mut offset = 0;
        let mut voices = Vec::<Voice>::new();
        for _ in 0..VOICE_COUNT {
            let packed_voice_data = &data[offset..offset + 128];
            let voice_data = Voice::unpack(packed_voice_data);
            let voice = Voice::from_bytes(&voice_data).expect("valid voice data");
            voices.push(voice);
            offset += 128;
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

    const DATA_SIZE: usize = 4096;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cartridge_length() {
        let cartridge = Cartridge::default();
        assert_eq!(cartridge.to_bytes().len(), 4096);
    }
}
