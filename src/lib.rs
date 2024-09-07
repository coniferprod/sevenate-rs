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

// Here is a trick learned from "Programming Rust" 2nd Ed., p. 280.
// Define associated consts in a trait, but don't give them a value.
// Let the implementor of the trait do that.
pub trait Ranged {
    const MIN: i32;
    const MAX: i32;
    const DEFAULT: i32;

    fn new(value: i32) -> Self;
    fn value(&self) -> i32;
    fn is_valid(value: i32) -> bool;
    fn random() -> Self;
}
