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
