pub mod dx7;

use std::fmt;

/// Error type for parsing data from MIDI System Exclusive bytes.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ParseError {
    InvalidLength(u32, u32),  // actual, expected
    InvalidChecksum(u8, u8),  // actual, expected
    InvalidData(u32),  // offset in data
    Unidentified,  // can't identify this kind
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            ParseError::InvalidLength(actual, expected) => format!("Got {} bytes of data, expected {} bytes.", actual, expected),
            ParseError::InvalidChecksum(actual, expected) => format!("Computed checksum was {}H, expected {}H.", actual, expected),
            ParseError::InvalidData(offset) => format!("Invalid data at offset {}.", offset),
            ParseError::Unidentified => String::from("Unable to identify this System Exclusive file."),
        })
    }
}

/// An `i32` constrained to a range.
pub trait Ranged {
    fn first() -> i32;
    fn last() -> i32;

    fn new(value: i32) -> Self;
    fn value(&self) -> i32;
    fn is_valid(value: i32) -> bool;
    fn random_value() -> Self;
}

// Private struct for saving the endpoints of a range.
// Intended as a convenience for types implementing `Ranged`.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Interval {
    first: i32,
    last: i32,
}

// Here is a trick learned from "Programming Rust" 2nd Ed., p. 280.
// Define associated consts in a trait, but don't give them a value.
// Let the implementor of the trait do that.
pub trait Ranged2 {
    const MIN: i32;
    const MAX: i32;
    const DEFAULT: i32;

    fn new(value: i32) -> Self;
    fn value(&self) -> i32;
    fn is_valid(value: i32) -> bool;
    fn random_value() -> Self;
}

// Let's do a quick newtype and implement the above trait.
pub struct Algorithm2(i32);

use rand::Rng;

impl Ranged2 for Algorithm2 {
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

    fn random_value() -> Self {
        let mut rng = rand::thread_rng();
        Self::new(rng.gen_range(Self::MIN..=Self::MAX))
    }

}

impl Default for Algorithm2 {
    fn default() -> Self {
        Self::new(Self::DEFAULT)
    }
}
