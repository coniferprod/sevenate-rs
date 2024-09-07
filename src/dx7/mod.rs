use std::fmt;
use std::convert::{
    From,
    TryFrom
};
use rand::Rng;
use bit::BitIndex;

use crate::{
    Ranged,
    ParseError
};

use crate::dx7::operator::{
    Operator,
    OperatorMode,
    ScalingCurve,
    Key,
    KeyboardLevelScaling,
    Scaling
};

use crate::dx7::envelope::{
    Envelope,
    Rate
};

use crate::dx7::sysex::{
    SystemExclusiveData,
    checksum,
};

use crate::dx7::voice::Voice;

use crate::dx7::lfo::{
    LfoWaveform,
    Lfo
};

pub mod voice;
pub mod cartridge;
pub mod operator;
pub mod lfo;
pub mod envelope;
pub mod sysex;

/// MIDI channel (1...16)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MIDIChannel(i32);

impl MIDIChannel {
    pub fn as_byte(&self) -> u8 {
        (self.0 - 1) as u8  // adjust to 0...15 for SysEx
    }
}

impl Ranged for MIDIChannel {
    const MIN: i32 = 1;
    const MAX: i32 = 16;
    const DEFAULT: i32 = Self::MIN;

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

// NOTE: Implementing TryFrom means that TryInto is implemented as well.

impl TryFrom<u8> for MIDIChannel {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let v: i32 = (value + 1).into(); // make into 1...16
        if MIDIChannel::is_valid(v) {
            Ok(MIDIChannel::new(v))
        }
        else {
            Err("Bad MIDI channel value")
        }
    }
}

pub static ALGORITHM_DIAGRAMS: [&str; 32] = include!("algorithms.in");

/// Algorithm (1...32)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Algorithm(i32);

impl Ranged for Algorithm {
    const MIN: i32 = 1;
    const MAX: i32 = 32;
    const DEFAULT: i32 = 32;

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

impl Default for Algorithm {
    fn default() -> Self {
        Self::new(Self::DEFAULT)
    }
}

impl Algorithm {
    pub fn as_byte(&self) -> u8 {
        (self.0 - 1) as u8  // adjust to 0...31 for SysEx
    }
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}:\n{}",
            self.0,
            ALGORITHM_DIAGRAMS[(self.0 as usize) - 1])
    }
}

impl TryFrom<u8> for Algorithm {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let v: i32 = (value + 1).into(); // make into 1...32
        if Algorithm::is_valid(v) {
            Ok(Algorithm::new(v))
        }
        else {
            Err("bad algorithm value")
        }
    }
}

/// Detune (-7...+7), represented in SysEx as 0...14.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Detune(i32);

impl Detune {
    pub fn as_byte(&self) -> u8 {
        (self.0 + 7) as u8  // adjust for SysEx
    }
}

impl Default for Detune {
    fn default() -> Detune {
        Detune::new(Detune::DEFAULT)
    }
}

impl fmt::Display for Detune {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for Detune {
    fn from(item: u8) -> Self {
        Detune::new((item - 7) as i32)
    }
}

impl Ranged for Detune {
    const MIN: i32 = -7;
    const MAX: i32 = 7;
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

/// Coarse (0...31).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Coarse(i32);

impl Coarse {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl Default for Coarse {
    fn default() -> Coarse {
        Coarse::new(Coarse::DEFAULT)
    }
}

impl fmt::Display for Coarse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for Coarse {
    fn from(item: u8) -> Self {
        Coarse::new(item as i32)
    }
}

impl Ranged for Coarse {
    const MIN: i32 = 0;
    const MAX: i32 = 31;
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

/// Depth (0...7) for keyboard rate scaling,
/// key velocity sensitivity, feedback,
/// pitch mod sensitivity.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Depth(i32);

impl Depth {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl Ranged for Depth {
    const MIN: i32 = 0;
    const MAX: i32 = 7;
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

impl Default for Depth {
    fn default() -> Depth {
        Depth::new(Depth::DEFAULT)
    }
}

/// Key transpose in semitones (-24...24).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Transpose(i32);

impl Transpose {
    pub fn as_byte(&self) -> u8 {
        // Convert to the range 0...48
        (self.0 + 24) as u8
    }
}

impl Default for Transpose {
    fn default() -> Transpose {
        Transpose::new(Transpose::DEFAULT)
    }
}

impl fmt::Display for Transpose {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for Transpose {
    /// Makes a key transpose from a System Exclusive data byte.
    fn from(item: u8) -> Self {
        // SysEx value is 0...48, corresponding to four octaves
        // with 12 semitones each)
        let semitones = item as i32 - 24;  // bring to range -24...24
        Transpose::new(semitones).try_into().unwrap()
    }
}

impl Ranged for Transpose {
    const MIN: i32 = -24;
    const MAX: i32 = 24;
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

/// Amplitude modulation sensitivity (0...3)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Sensitivity(i32);

impl Sensitivity {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl Default for Sensitivity {
    fn default() -> Sensitivity {
        Sensitivity::new(Sensitivity::DEFAULT)
    }
}

impl fmt::Display for Sensitivity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for Sensitivity {
    fn from(item: u8) -> Self {
        Sensitivity::new(item as i32)
    }
}

impl Ranged for Sensitivity {
    const MIN: i32 = 0;
    const MAX: i32 = 3;
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

/// Envelope level (or operator output level) (0...99)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Level(i32);

impl Level {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl Default for Level {
    fn default() -> Level {
        Level::new(0)
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for Level {
    fn from(item: u8) -> Self {
        Level::new(item as i32)
    }
}

impl Ranged for Level {
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

//
// Utilities for creating voices and cartridges
//

/// Makes a new voice based on the "BRASS1" settings in the DX7 manual.
pub fn make_brass1() -> Voice {
    let kbd_level_scaling = KeyboardLevelScaling {
        breakpoint: Key::new(60 - 21),
        left: Scaling { depth: Level::new(0), curve: ScalingCurve::lin_pos() },
        right: Scaling { depth: Level::new(0), curve: ScalingCurve::lin_pos() },
    };

    // Make one operator and then specify the differences to the others.
    let op = Operator {
        key_vel_sens: Depth::new(2),
        ..Operator::new()
    };

    let op6 = Operator {
        eg: Envelope::new_rate_level(
            [Rate::new(49), Rate::new(99), Rate::new(28), Rate::new(68)],
            [Level::new(98), Level::new(98), Level::new(91), Level::new(0)]),
        kbd_level_scaling: KeyboardLevelScaling {
            left: Scaling { depth: Level::new(54), curve: ScalingCurve::exp_neg() },
            right: Scaling { depth: Level::new(50), curve: ScalingCurve::exp_neg() },
            ..kbd_level_scaling
        },
        kbd_rate_scaling: Depth::new(4),
        output_level: Level::new(82),
        ..op
    };

    let op5 = Operator {
        eg: Envelope::new_rate_level(
            [Rate::new(77), Rate::new(36), Rate::new(41), Rate::new(71)],
            [Level::new(99), Level::new(98), Level::new(98), Level::new(0)]),
        kbd_level_scaling,
        output_level: Level::new(98),
        detune: Detune::new(1),
        ..op
    };

    let op4 = Operator {
        eg: op5.eg.clone(),
        kbd_level_scaling,
        output_level: Level::new(99),
        ..op
    };

    let op3 = Operator {
        eg: Envelope::new_rate_level(
            [Rate::new(77), Rate::new(76), Rate::new(82), Rate::new(71)],
            [Level::new(99), Level::new(98), Level::new(98), Level::new(0)]),
        kbd_level_scaling,
        output_level: Level::new(99),
        detune: Detune::new(-2),
        ..op
    };

    let op2 = Operator {
        eg: Envelope::new_rate_level(
            [Rate::new(62), Rate::new(51), Rate::new(29), Rate::new(71)],
            [Level::new(82), Level::new(95), Level::new(96), Level::new(0)]),
        kbd_level_scaling: KeyboardLevelScaling {
            breakpoint: Key::new(48 - 21),
            left: Scaling { depth: Level::new(0), curve: ScalingCurve::lin_pos() },
            right: Scaling { depth: Level::new(7), curve: ScalingCurve::exp_neg() },
        },
        key_vel_sens: Depth::new(0),
        output_level: Level::new(86),
        coarse: Coarse::default(),
        detune: Detune::new(7),
        ..op
    };

    let op1 = Operator {
        eg: Envelope::new_rate_level(
            [Rate::new(72), Rate::new(76), Rate::new(99), Rate::new(71)],
            [Level::new(99), Level::new(88), Level::new(96), Level::new(0)]),
        kbd_level_scaling: KeyboardLevelScaling {
            right: Scaling { depth: Level::new(14), curve: ScalingCurve::lin_pos() },
            ..kbd_level_scaling
        },
        key_vel_sens: Depth::new(0),
        output_level: Level::new(98),
        coarse: Coarse::default(),
        detune: Detune::new(7),
        ..op
    };

    Voice {
        operators: [op1, op2, op3, op4, op5, op6],
        peg: Envelope::new_rate_level(
            [Rate::new(84), Rate::new(95), Rate::new(95), Rate::new(60)],
            [Level::new(50), Level::new(50), Level::new(50), Level::new(50)]),
        alg: Algorithm::new(22),
        feedback: Depth::new(7),
        osc_sync: true,
        lfo: Lfo {
            speed: Level::new(37),
            delay: Level::new(0),
            pmd: Level::new(5),
            amd: Level::new(0),
            sync: false,
            waveform: LfoWaveform::Sine,
        },
        pitch_mod_sens: Depth::new(3),
        transpose: Transpose::new(0),
        name: "BRASS   1 ".to_string(),
    }
}

/// Makes an initialized voice. The defaults are as described in
/// Howard Massey's "The Complete DX7", Appendix B.
pub fn make_init_voice() -> Voice {
    let init_eg = Envelope::new();

    let init_op1 = Operator {
        eg: init_eg.clone(),
        kbd_level_scaling: KeyboardLevelScaling::new(),
        kbd_rate_scaling: Depth::new(0),
        amp_mod_sens: Sensitivity::new(0),
        key_vel_sens: Depth::new(0),
        output_level: Level::new(99),
        mode: OperatorMode::Ratio,
        coarse: Coarse::new(1),
        fine: Level::new(0),
        detune: Detune::default(),
    };

    // Operators 2...6 are identical to operator 1 except they
    // have their output level set to zero.
    let init_op_rest = Operator {
        output_level: Level::new(0),
        ..init_op1
    };

    Voice {
        operators: [
            init_op1.clone(),
            init_op_rest.clone(),
            init_op_rest.clone(),
            init_op_rest.clone(),
            init_op_rest.clone(),
            init_op_rest.clone(),
        ],
        peg: Envelope::new_rate_level(
            [Rate::new(99), Rate::new(99), Rate::new(99), Rate::new(99)],
            [Level::new(50), Level::new(50), Level::new(50), Level::new(50)]),
        alg: Algorithm::new(1),
        feedback: Depth::new(0),
        osc_sync: true, // osc key sync = on
        lfo: Lfo {
            speed: Level::new(35),
            delay: Level::new(0),
            pmd: Level::new(0),
            amd: Level::new(0),
            sync: true,
            waveform: LfoWaveform::Triangle,
        },
        pitch_mod_sens: Depth::new(3),
        transpose: Transpose::new(0),
        name: "INIT VOICE".to_string(),
    }
}

// Finds the first offset where the two vectors differ.
// Returns None if no differences are found, or if the vectors
// are different lengths, Some<usize> with the offset otherwise.
pub fn first_different_offset(v1: &[u8], v2: &[u8]) -> Option<usize> {
    if v1.len() != v2.len() {
        return None;
    }

    let mut offset = 0;
    for i in 0..v1.len() {
        if v1[i] != v2[i] {
            return Some(offset);
        }
        else {
            offset += 1;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_bit_range() {
        let b: u8 = 0b00110000;

        // If this succeeds, the range upper bound is not included,
        // i.e. 4..6 means bits 4 and 5.
        assert_eq!(b.bit_range(4..6), 0b11);
    }

    #[test]
    #[should_panic(expected = "expected value in range")]
    fn test_invalid_new_panics() {
        let alg = Algorithm::new(0);
        assert_eq!(alg.value(), 0);
    }

    #[test]
    fn test_valid_new_succeeds() {
        let alg = Algorithm::new(32);
        assert_eq!(alg.value(), 32);
    }

    #[test]
    fn test_eg_to_bytes() {
        let eg = Envelope {
            rates: [Rate::new(64), Rate::new(64), Rate::new(64), Rate::new(64)],
            levels: [Level::new(32), Level::new(32), Level::new(32), Level::new(32)]
        };
        assert_eq!(eg.to_bytes(), vec![64u8, 64, 64, 64, 32, 32, 32, 32]);
    }

    #[test]
    fn test_scaling_curve_exp_pos_to_bytes() {
        assert_eq!(ScalingCurve::exp_pos().as_byte(), 2);
    }

    #[test]
    fn test_scaling_curve_exp_neg_to_bytes() {
        assert_eq!(ScalingCurve::exp_neg().as_byte(), 1);
    }

    #[test]
    fn test_scaling_curve_lin_pos_to_bytes() {
        assert_eq!(ScalingCurve::lin_pos().as_byte(), 3);
    }

    #[test]
    fn test_scaling_curve_lin_neg_to_bytes() {
        assert_eq!(ScalingCurve::lin_neg().as_byte(), 0);
    }


    #[test]
    fn test_bulk_b111() {
        let sync = true;
        let feedback = 7u8;
        let expected = 0x0fu8;
        let actual = feedback | ((if sync { 1 } else { 0 }) << 3);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_bulk_b116() {
        let sync = false;
        let wave = LfoWaveform::Sine;
        let pms = 3u8;
        let mut actual: u8 = if sync { 1 } else { 0 };
        actual.set_bit_range(1..4, wave as u8);
        actual.set_bit_range(4..7, pms);
        assert_eq!(actual, 0x38);
    }

    #[test]
    fn test_transpose_from_byte() {
        let zero = Transpose::from(48);  // from SysEx byte
        assert_eq!(zero.value(), 24);
    }

    #[test]
    fn test_transpose_from_byte_minus_two() {
        let minus_two_octaves = Transpose::from(0);  // from SysEx byte
        assert_eq!(minus_two_octaves.value(), -24);
    }

    #[test]
    fn test_transpose_from_byte_minus_one() {
        let minus_one_octave = Transpose::from(12);  // from SysEx byte
        assert_eq!(minus_one_octave.value(), -12);
    }

    #[test]
    fn test_transpose_as_byte() {
        let none = Transpose::new(0);  // no transpose
        assert_eq!(none.as_byte(), 24);

        let plus_two = Transpose::new(24);
        assert_eq!(plus_two.as_byte(), 48)
    }
}
