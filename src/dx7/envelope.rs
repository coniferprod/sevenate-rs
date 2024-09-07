use std::fmt;
use rand::Rng;

use crate::{
    Ranged,
    ParseError
};
use crate::dx7::Level;
use crate::dx7::sysex::SystemExclusiveData;

/// Envelope rate (0...99)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Rate(i32);

impl Rate {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl Default for Rate {
    fn default() -> Rate {
        Rate::new(Rate::DEFAULT)
    }
}

impl fmt::Display for Rate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for Rate {
    fn from(item: u8) -> Self {
        Rate::new(item as i32)
    }
}

impl Ranged for Rate {
    const MIN: i32 = 0;
    const MAX: i32 = 99;
    const DEFAULT: i32 = 0;

    fn new(value: i32) -> Self {
        if Self::is_valid(value) {
            Self(value)
        }
        else {
            panic!("expected value in range {}...{}, got {}",
                Self::MIN, Self::MAX, value);
        }
    }

    fn value(&self) -> i32 { self.0 }

    fn is_valid(value: i32) -> bool {
        value >= Self::MIN && value <= Self::MAX
    }

    fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self::new(rng.gen_range(Self::MIN..=Self::MAX))
    }
}

pub type Rates = [Rate; 4];
pub type Levels = [Level; 4];

/// Envelope generator.
#[derive(Debug, Clone, Copy)]
pub struct Envelope {
    pub rates: Rates,
    pub levels: Levels,
}

impl Envelope {
    /// Creates a new EG with the DX7 voice defaults.
    pub fn new() -> Self {
        Envelope {
            rates: [Rate::new(99), Rate::new(99), Rate::new(99), Rate::new(99)],
            levels: [Level::new(99), Level::new(99), Level::new(99), Level::new(0)]
        }
    }

    /// Makes a new EG with rates and levels.
    pub fn new_rate_level(rates: Rates, levels: Levels) -> Self {
        Self { rates, levels }
    }

    pub fn new_rate_level_int(rates: [i32; 4], levels: [i32; 4]) -> Self {
        let mut r: [Rate; 4] = [Rate::default(); 4];
        let mut l: [Level; 4] = [Level::default(); 4];
        for i in 0..rates.len() {
            r[i] = Rate::new(rates[i]);
            l[i] = Level::new(levels[i]);
        }
        Self { rates: r, levels: l }
    }

    /*
    From the Yamaha DX7 Operation Manual (p. 51):
    "You can simulate an ADSR if you set the envelope as follows:
    L1=99, L2=99, L4=0, and R2=99.
    With these settings, then R1 becomes Attack time, R3 is Decay
    time, L3 is Sustain level, and R4 is Release time."
    */

    /// Makes a new ADSR-style envelope.
    pub fn adsr(attack: Rate, decay: Rate, sustain: Level, release: Rate) -> Self {
        Envelope::new_rate_level(
            [attack, Rate::new(99), decay, release],
            [Level::new(99), Level::new(99), sustain, Level::new(0)]
        )
    }

    pub fn adsr_int(attack: i32, decay: i32, sustain: i32, release: i32) -> Self {
        Envelope::new_rate_level_int(
            [attack, 99, decay, release],
            [99, 99, sustain, 0]
        )
    }

    /// Makes a new EG with random rates and levels.
    pub fn random() -> Self {
        Self {
            rates: [Rate::random(), Rate::random(), Rate::random(), Rate::random()],
            levels: [Level::random(), Level::random(), Level::random(), Level::random()],
        }
    }
}

impl fmt::Display for Envelope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "R1={} L1={} R2={} L2={} R3={} L3={} R4={} L4={}",
            self.rates[0].value(), self.levels[0].value(),
            self.rates[1].value(), self.levels[1].value(),
            self.rates[2].value(), self.levels[2].value(),
            self.rates[3].value(), self.levels[3].value())
    }
}

impl SystemExclusiveData for Envelope {
    /// Makes an envelope generator from relevant SysEx message bytes.
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        Ok(Envelope::new_rate_level_int(
            [data[0].into(), data[1].into(), data[2].into(), data[3].into()],
            [data[4].into(), data[5].into(), data[6].into(), data[7].into()]
        ))
    }

    /// Gets the SysEx bytes of this EG.
    fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.rates[0].as_byte(),
            self.rates[1].as_byte(),
            self.rates[2].as_byte(),
            self.rates[3].as_byte(),
            self.levels[0].as_byte(),
            self.levels[1].as_byte(),
            self.levels[2].as_byte(),
            self.levels[3].as_byte()
        ]
    }

    const DATA_SIZE: usize = 8;
}
