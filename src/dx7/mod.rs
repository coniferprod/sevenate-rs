use std::fmt;

use rand::Rng;
use syxpack::{
    Ranged,
    ranged_impl,
    Encoding,
};

pub mod voice;
pub mod cartridge;
pub mod operator;
pub mod lfo;
pub mod envelope;
pub mod sysex;

/// Algorithm (1...32)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Algorithm(i32);

ranged_impl!(Algorithm, 1, 32, 32);

impl Encoding for Algorithm {
    fn decode(b: u8) -> i32 {
        (b as i32) + 1  // adjust to 1...32
    }

    fn encode(&self) -> u8 {
        (self.value() - 1) as u8 // adjust to 0...31 for SysEx
    }
}

/// Detune (-7...+7), represented in SysEx as 0...14.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Detune(i32);

ranged_impl!(Detune, -7, 7, 0);

impl Encoding for Detune {
    fn decode(b: u8) -> i32 {
        (b as i32) - 7  // adjust to -7...+7
    }

    fn encode(&self) -> u8 {
        (self.value() + 7) as u8  // adjust to 0...14 for SysEx
    }
}

/// Coarse (0...31).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Coarse(i32);

ranged_impl!(Coarse, 0, 31, 0);

impl Encoding for Coarse { }  // identity mapping, no adjustment needed

/// Depth (0...7) for keyboard rate scaling,
/// key velocity sensitivity, feedback,
/// pitch mod sensitivity.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Depth(i32);

ranged_impl!(Depth, 0, 7, 0);

impl Encoding for Depth { } // identity mapping, no adjustment needed

/// Key transpose in semitones (-24...+24, or two octaves).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Transpose(i32);

ranged_impl!(Transpose, -24, 24, 0);

impl Encoding for Transpose {
    fn decode(b: u8) -> i32 {
        // SysEx value is 0...48, corresponding to four octaves
        // with 12 semitones each)
        (b as i32) - 24  // adjust to -24...+24
    }

    fn encode(&self) -> u8 {
        (self.value() + 24) as u8  // adjust to 0...48 for SysEx
    }
}

/// Amplitude modulation sensitivity (0...3)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Sensitivity(i32);

ranged_impl!(Sensitivity, 0, 3, 0);

impl Encoding for Sensitivity { } // identity mapping, no adjustment needed

/// Envelope level (or operator output level) (0...99)
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Level(i32);

ranged_impl!(Level, 0, 99, 0);

impl Encoding for Level { } // identity mapping, no adjustment needed

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
    use syxpack::parse_or_default;

    use super::*;
    use crate::dx7::lfo::*;

    #[test]
    fn test_bit_range() {
        let b: u8 = 0b00110000;

        // If this succeeds, the range upper bound is not included,
        // i.e. 4..6 means bits 4 and 5.
        assert_eq!(b.bit_range(4..6), 0b11);
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
        let zero = parse_or_default::<Transpose>(48);  // from SysEx byte
        assert_eq!(zero.value(), 24);
    }

    #[test]
    fn test_transpose_from_byte_minus_two() {
        let minus_two_octaves = parse_or_default::<Transpose>(0);  // from SysEx byte
        assert_eq!(minus_two_octaves.value(), -24);
    }

    #[test]
    fn test_transpose_from_byte_minus_one() {
        let minus_one_octave = parse_or_default::<Transpose>(24);  // from SysEx byte
        assert_eq!(minus_one_octave.value(), 0);
    }

    #[test]
    fn test_transpose_as_byte() {
        let none = Transpose::new(0);  // no transpose
        assert_eq!(none.encode(), 24);

        let plus_two = Transpose::new(24);
        assert_eq!(plus_two.encode(), 48)
    }
}

/// ASCII art diagrams for the DX7 algorithms.
pub static ALGORITHM_DIAGRAMS: [&str; 32] = [
// Algorithm #1:
"
         +---+
       +-+-+ |
       | 6 | |
       +-+-+ |
         |---+
       +-+-+
       | 5 |
       +-+-+
         |
+---+  +-+-+
| 2 |  | 4 |
+-+-+  +-+-+
  |      |
+-+-+  +-+-+
| 1 |  | 3 |
+-+-+  +-+-+
  |      |
  +------+
",

// Algorithm #2:
"
         +---+
         | 6 |
         +-+-+
           |
         +-+-+
         | 5 |
         +-+-+
+---+      |
| +-+-+  +-+-+
| | 2 |  | 4 |
| +-+-+  +-+-+
+---|      |
  +-+-+  +-+-+
  | 1 |  | 3 |
  +-+-+  +-+-+
    |      |
    +------+
",

// Algorithm #3:
"
         +---+
+---+  +-+-+ |
| 3 |  | 6 | |
+-+-+  +-+-+ |
  |      +---+
+-+-+  +-+-+
| 2 |  | 5 |
+-+-+  +-+-+
  |      |
+-+-+  +-+-+
| 1 |  | 4 |
+-+-+  +-+-+
  |      |
  +------+
",

// Algorithm #4:
"
         +---+
+---+  +-+-+ |
| 3 |  | 6 | |
+-+-+  +-+-+ |
  |      |   |
+-+-+  +-+-+ |
| 2 |  | 5 | |
+-+-+  +-+-+ |
  |      |   |
+-+-+  +-+-+ |
| 1 |  | 4 | |
+-+-+  +-+-+ |
  |      +---+
  +------+
",

// Algorithm #5:
"
                +---+
+---+  +---+  +-+-+ |
| 2 |  | 4 |  | 6 | |
+-+-+  +-+-+  +-+-+ |
  |      |      +---+
+-+-+  +-+-+  +-+-+
| 1 |  | 3 |  | 5 |
+-+-+  +-+-+  +-+-+
  |      |      |
  +------+------+
",

// Algorithm #6:
"
                +---+
+---+  +---+  +-+-+ |
| 2 |  | 4 |  | 6 | |
+-+-+  +-+-+  +-+-+ |
  |      |      |   |
+-+-+  +-+-+  +-+-+ |
| 1 |  | 3 |  | 5 | |
+-+-+  +-+-+  +-+-+ |
  |      |      +---+
  +------+------+
",

// Algorithm #7:
"
              +---+
            +-+-+ |
            | 6 | |
            +-+-+ |
              +---+
+---+ +---+ +-+-+
| 2 | | 4 | | 5 |
+-+-+ +-+-+ +---+
  |     |  /
+-+-+ +-+-+
| 1 | | 3 |
+-+-+ +-+-+
  |     |
  +-----+
",

// Algorithm #8:
"
              +---+
              | 6 |
              +-+-+
      +---+     |
+---+ | +-+-+ +-+-+
| 2 | | | 4 | | 5 |
+-+-+ | +-+-+ +---+
  |   +---+  /
+-+-+   +-+-+
| 1 |   | 3 |
+-+-+   +-+-+
  |       |
  +-------+
",

// Algorithm #9:
"
              +---+
              | 6 |
              +-+-+
+---+           |
| +-+-+ +-+-+ +-+-+
| | 2 | | 4 | | 5 |
| +-+-+ +-+-+ +-+-+
+---+     |  /
  +-+-+ +-+-+
  | 1 | | 3 |
  +-+-+ +-+-+
    |     |
    +-----+
",

// Algorithm #10:
r"
              +---+
            +-+-+ |
            | 3 | |
            +-+-+ |
              +---+
+---+ +---+ +-+-+
| 5 | | 6 | | 2 |
+---+ +-+-+ +-+-+
     \  |     |
      +-+-+ +-+-+
      | 4 | | 1 |
      +-+-+ +-+-+
        |     |
        +-----+
",

// Algorithm #11:
r"
            +-+-+
            | 3 |
            +-+-+
        +--+  |
+---+ +-+-+|+-+-+
| 5 | | 6 ||| 2 |
+---+ +-+-+|+-+-+
     \  +--+  |
      +-+-+ +-+-+
      | 4 | | 1 |
      +-+-+ +-+-+
        |     |
        +-----+
",

// Algorithm #12:
r"
                    +---+
+---+ +---+ +---+ +-+-+ |
| 4 | | 5 | | 6 | | 2 | |
+---+ +-+-+ +---+ +-+-+ |
     \  |  /        +---+
      +-+-+       +-+-+
      | 3 |       | 1 |
      +-+-+       +-+-+
        +-----------+
",

// Algorithm #13:
r"
              +--+
+---+ +---+ +-+-+|+-+-+
| 4 | | 5 | | 6 ||| 2 |
+---+ +-+-+ +-+-+|+-+-+
     \  |  /  +--+  |
      +-+-+       +-+-+
      | 3 |       | 1 |
      +-+-+       +-+-+
        +-----------+
",

// Algorithm #14:
r"
        +---+
+---+ +-+-+ |
| 5 | | 6 | |
+---+ +-+-+ |
     \  +---+
+-+-+ +-+-+
| 2 | | 4 |
+-+-+ +-+-+
  |     |
+-+-+ +-+-+
| 1 | | 3 |
+---+ +---+
",

// Algorithm #15:
r"
  +---+ +-+-+
  | 5 | | 6 |
  +---+ +-+-+
+---+  \  |
| +-+-+ +-+-+
| | 2 | | 4 |
| +-+-+ +-+-+
+---+     |
  +-+-+ +-+-+
  | 1 | | 3 |
  +-+-+ +-+-+
    |     |
    +-----+
",

// Algorithm #16:
r"
              +---+
      +---+ +-+-+ |
      | 4 | | 6 | |
      +-+-+ +-+-+ |
        |     +---+
+---+ +-+-+ +-+-+
| 2 | | 3 | | 5 |
+---+ +-+-+ +---+
     \  |  /
      +-+-+
      | 1 |
      +---+
",

// Algorithm #17:
r"
      +---+ +-+-+
      | 4 | | 6 |
      +-+-+ +-+-+
  +--+  |     |
+-+-+|+-+-+ +-+-+
| 2 ||| 3 | | 5 |
+---+|+-+-+ +---+
     \  |  /
      +-+-+
      | 1 |
      +---+
",

// Algorithm #18:
r"
                 +--+--+
                 |  6  |
                 +--+--+
                    |
                 +--+--+
                 |  5  |
                 +--+--+
           +----+   |
+--+--+ +--+--+ |+--+--+
|  2  | |  3  | ||  4  |
+-----+ +--+--+ |+-----+
       \   |---//
        +--+---+
        |  1   |
        +------+
",

// Algorithm #19:
r"
+---+
| 3 |
+-+-+
  |     +---+
+-+-+ +-+-+ |
| 2 | | 6 | |
+-+-+ +-+-+ |
  |     |  \|
+-+-+ +-+-+ +---+
| 1 | | 4 | | 5 |
+-+-+ +-+-+ +-+-+
  |     |     |
  +-----+-----+
",

// Algorithm #20:
r"
  +---+
+-+-+ |     +---+ +---+
| 3 | |     | 5 | | 6 |
+-+-+ |     +---+ +-+-+
  |  \|          \  |
+-+-+ +-+-+       +-+-+
| 1 | | 2 |       | 4 |
+-+-+ +-+-+       +-+-+
  |     |           |
  +-----+-----------+
",

// Algorithm #21:
r"
  +---+
+-+-+ |     +---+
| 3 | |     | 6 |
+-+-+ |     +-+-+
  |  \|       |  \
+-+-+ +---+ +-+-+ +---+
| 1 | | 2 | | 4 | | 5 |
+-+-+ +-+-+ +-+-+ +-+-+
  |     |     |     |
  +-----+-----+-----+
",

// Algorithm #22:
r"
              +---+
+-+-+       +-+-+ |
| 2 |       | 6 | |
+-+-+       +-+-+ |
  |        /  |  \|
+-+-+ +---+ +-+-+ +---+
| 1 | | 3 | | 4 | | 5 |
+-+-+ +-+-+ +-+-+ +-+-+
  |     |     |     |
  +-----+-----+-----+
",

// Algorithm #23:
r"
              +---+
      +-+-+ +-+-+ |
      | 3 | | 6 | |
      +-+-+ +-+-+ |
        |     |  \|
+---+ +-+-+ +-+-+ +---+
| 1 | | 2 | | 4 | | 5 |
+-+-+ +-+-+ +-+-+ +-+-+
  |     |     |     |
  +-----+-----+-----+
",

// Algorithm #24:
r"
                    +---+
                  +-+-+ |
                  | 6 | |
                  +-+-+ |
                 /  |  \|
+---+ +---+ +---+ +-+-+ +-+-+
| 1 | | 2 | | 3 | | 4 | | 5 |
+-+-+ +-+-+ +-+-+ +-+-+ +-+-+
  |     |     |     |     |
  +-----+-----+-----+-----+
",

// Algorithm #25:
r"
                    +---+
                  +-+-+ |
                  | 6 | |
                  +-+-+ |
                    |  \|
+---+ +---+ +---+ +-+-+ +---+
| 1 | | 2 | | 3 | | 4 | | 5 |
+-+-+ +-+-+ +-+-+ +-+-+ +-+-+
  |     |     |     |     |
  +-----+-----+-----+-----+
",

// Algorithm #26:
r"
                       +---+
       +-+-+   +---+ +-+-+ |
       | 3 |   | 5 | | 6 | |
       +-+-+   +---+ +-+-+ |
         |          \  +---+
+---+  +-+-+         +-+-+
| 1 |  | 2 |         | 4 |
+-+-+  +-+-+         +-+-+
  |      |             |
  +------+------+------+
",

// Algorithm #27:
r"
         +---+
       +-+-+ | +---+ +---+
       | 3 | | | 5 | | 6 |
       +-+-+ | +---+ +-+-+
         +---+      \  |
+---+  +-+-+         +-+-+
| 1 |  | 2 |         | 4 |
+-+-+  +-+-+         +-+-+
  |      |             |
  +------+------+------+
",

// Algorithm #28:
"
       +---+
       | 5 +-+
       +-+-+ |
         |---+
+---+  +-+-+
| 2 |  | 4 |
+-+-+  +-+-+
  |      |
+-+-+  +-+-+  +---+
| 1 |  | 3 |  | 6 |
+-+-+  +-+-+  +-+-+
  |      |      |
  +------+------+
",

// Algorithm #29:
"
                       +---+
              +-+-+  +-+-+ |
              | 4 |  | 6 | |
              +-+-+  +-+-+ |
                |      |---+
+---+  +---+  +-+-+  +-+-+
| 1 |  | 2 |  | 3 |  | 5 |
+-+-+  +-+-+  +-+-+  +-+-+
  |      |      |      |
  +------+------+------+
",

// Algorithm #30:
"
                +---+
              +-+-+ |
              | 5 | |
              +-+-+ |
                |---+
              +-+-+
              | 4 |
              +-+-+
                |
+---+  +---+  +-+-+  +---+
| 1 |  | 2 |  | 3 |  | 6 |
+-+-+  +-+-+  +-+-+  +-+-+
  |      |      |      |
  +------+------+------+
",

// Algorithm #31:
"
                            +---+
                            | 6 +-+
                            +-+-+ |
                              |---+
+---+  +---+  +---+  +---+  +-+-+
| 1 |  | 2 |  | 3 |  | 4 |  | 5 |
+-+-+  +-+-+  +-+-+  +-+-+  +-+-+
  |      |      |      |      |
  +------+------+------+------+
",

// Algorithm #32:
"
                                     +---+
+---+  +---+  +---+  +---+  +---+  +-+-+ |
| 1 |  | 2 |  | 3 |  | 4 |  | 5 |  | 6 | |
+-+-+  +-+-+  +-+-+  +-+-+  +-+-+  +-+-+ |
  |      |      |      |      |      |---+
  +------+------+------+------+------+
",
];
