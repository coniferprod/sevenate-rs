use std::fmt;
use bit::BitIndex;
use log::warn;

use crate::{
    ParseError,
    Ranged
};
use crate::dx7::Level;
use crate::dx7::sysex::SystemExclusiveData;

/// LFO waveform.
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum LfoWaveform {
    Triangle,
    SawDown,
    SawUp,
    Square,
    Sine,
    SampleAndHold,
}

/// LFO.
#[derive(Debug, Clone, Copy)]
pub struct Lfo {
    pub speed: Level,  // 0 ~ 99
    pub delay: Level,  // 0 ~ 99
    pub pmd: Level,    // 0 ~ 99
    pub amd: Level,    // 0 ~ 99
    pub sync: bool,
    pub waveform: LfoWaveform,
}

impl Lfo {
    /// Makes a new LFO initialized with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            speed: Level::new(35),
            delay: Level::new(0),
            pmd: Level::new(0),
            amd: Level::new(0),
            sync: true,
            waveform: LfoWaveform::Triangle,
        }
    }

    /// Makes a new LFO with random settings.
    pub fn new_random() -> Self {
        Self {
            speed: Level::random(),
            delay: Level::random(),
            pmd: Level::random(),
            amd: Level::random(),
            sync: true,
            waveform: LfoWaveform::Triangle,
        }
    }

    /// Unpacks LFO data from a cartridge.
    /// Returns the data in the same format as for a single voice.
    pub fn unpack(data: &[u8]) -> Vec<u8> {
        vec![
            data[0],  // LFO speed
            data[1],  // LFO delay
            data[2],  // LFO PMD
            data[3],  // LFO AMD
            if data[4].bit(0) { 1 } else { 0 },  // LFO sync
            data[4].bit_range(1..4), // LFO waveform
        ]
    }
}

impl fmt::Display for Lfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "speed = {}, delay = {}, PMD = {}, AMD = {}, sync = {}, waveform = {:?}",
            self.speed.value(),
            self.delay.value(),
            self.pmd.value(),
            self.amd.value(),
            self.sync,
            self.waveform)
    }
}

impl SystemExclusiveData for Lfo {
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        Ok(Lfo {
            speed: Level::new(data[0].into()),
            delay: Level::new(data[1].into()),
            pmd: Level::new(data[2].into()),
            amd: Level::new(data[3].into()),
            sync: if data[4] == 1 { true } else { false },
            waveform: match data[5] {
                0 => LfoWaveform::Triangle,
                1 => LfoWaveform::SawDown,
                2 => LfoWaveform::SawUp,
                3 => LfoWaveform::Square,
                4 => LfoWaveform::Sine,
                5 => LfoWaveform::SampleAndHold,
                _ => {
                    warn!("LFO waveform out of range: {}, setting to TRI", data[5]);
                    LfoWaveform::Triangle
                }
            },
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.speed.as_byte(),
            self.delay.as_byte(),
            self.pmd.as_byte(),
            self.amd.as_byte(),
            if self.sync { 1 } else { 0 },
            self.waveform as u8,
        ]
    }

    const DATA_SIZE: usize = 7;
}
