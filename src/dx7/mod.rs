use std::fmt;
use std::convert::{
    From,
    TryFrom
};
use rand::Rng;

use crate::{
    Ranged,
    ParseError
};

pub mod voice;
pub mod cartridge;
pub mod operator;
pub mod lfo;
pub mod envelope;
pub mod sysex;

/// ASCII art diagrams for the DX7 algorithms.
pub static ALGORITHM_DIAGRAMS: [&str; 32] = include!("algorithms.in");

/// Algorithm (1...32)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Algorithm(i32);

crate::ranged_impl!(Algorithm, 1, 32, 32);

impl Algorithm {
    pub fn as_byte(&self) -> u8 {
        (self.0 - 1) as u8  // adjust to 0...31 for SysEx
    }

    pub fn algorithm_display(&self) -> String {
        String::from(ALGORITHM_DIAGRAMS[(self.value() as usize) - 1])
    }
}

impl TryFrom<u8> for Algorithm {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let v: i32 = (value + 1).into(); // make into 1...32
        if Algorithm::contains(v) {
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

crate::ranged_impl!(Detune, -7, 7, 0);

impl Detune {
    pub fn as_byte(&self) -> u8 {
        (self.0 + 7) as u8  // adjust for SysEx
    }
}

impl From<u8> for Detune {
    fn from(item: u8) -> Self {
        Detune::new((item - 7) as i32)
    }
}

/// Coarse (0...31).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Coarse(i32);

crate::ranged_impl!(Coarse, 0, 31, 0);

impl Coarse {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl From<u8> for Coarse {
    fn from(item: u8) -> Self {
        Coarse::new(item as i32)
    }
}

/// Depth (0...7) for keyboard rate scaling,
/// key velocity sensitivity, feedback,
/// pitch mod sensitivity.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Depth(i32);

crate::ranged_impl!(Depth, 0, 7, 0);

impl Depth {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

/// Key transpose in octaves (-2...2).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Transpose(i32);

crate::ranged_impl!(Transpose, -2, 2, 0);

impl Transpose {
    pub fn as_byte(&self) -> u8 {
        // The transpose is expressed in octaves,
        // but the SysEx has it as semitones:
        let semitone_count = self.0 * 12;
        // So, -2 becomes -2*12 = -24,
        // and +2 becomes +2*12 = 24.

        // Convert to the range 0...48
        (semitone_count + 24) as u8
    }
}

impl From<u8> for Transpose {
    /// Makes a key transpose from a System Exclusive data byte.
    fn from(item: u8) -> Self {
        // SysEx value is 0...48, corresponding to four octaves
        // with 12 semitones each)
        let semitones = item as i32 - 24;  // bring to range -24...24
        let mut octave_count = semitones / 12;
        Transpose::new(octave_count).try_into().unwrap()
    }
}

/// Amplitude modulation sensitivity (0...3)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Sensitivity(i32);

crate::ranged_impl!(Sensitivity, 0, 3, 0);

impl Sensitivity {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl From<u8> for Sensitivity {
    fn from(item: u8) -> Self {
        Sensitivity::new(item as i32)
    }
}

/// Envelope level (or operator output level) (0...99)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Level(i32);

crate::ranged_impl!(Level, 0, 99, 0);

impl Level {
    pub fn as_byte(&self) -> u8 {
        self.0 as u8
    }
}

impl From<u8> for Level {
    fn from(item: u8) -> Self {
        Level::new(item as i32)
    }
}

// Finds the first offset where the two slices differ.
// Returns None if no differences are found, or if the slices
// are different lengths, Some<usize> with the offset otherwise.
pub fn compare_slices(v1: &[u8], v2: &[u8]) -> Option<usize> {
    if v1.len() != v2.len() {
        return None;
    }

    let mut offset = 0;
    for i in 0..v1.len() {
        if v1[i] != v2[i] {
            return Some(offset);
        }
        offset += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use bit::BitIndex;

    use super::*;
    use crate::dx7::operator::*;
    use crate::dx7::lfo::*;

    #[test]
    fn test_bit_range() {
        let b: u8 = 0b00110000;

        // If this succeeds, the range upper bound is not included,
        // i.e. 4..6 means bits 4 and 5.
        assert_eq!(b.bit_range(4..6), 0b11);
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
