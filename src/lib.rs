pub mod dx7;

use std::fmt;
use rand::Rng;

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
    const FIRST: i32;
    const LAST: i32;
    const DEFAULT: i32;

    fn new(value: i32) -> Self;
    fn value(&self) -> i32;
    fn contains(value: i32) -> bool;
    fn random() -> Self;
}

// The `ranged_impl` macro generates an implementation of the `Ranged` trait,
// along with implementations of the `Default` and `Display` traits based on
// the values supplied as parameters (type name, first, last, default).
#[macro_export]
macro_rules! ranged_impl {
    ($typ:ty, $first:expr, $last:expr, $default:expr) => {
        impl Ranged for $typ {
            const FIRST: i32 = $first;
            const LAST: i32 = $last;
            const DEFAULT: i32 = $default;

            fn new(value: i32) -> Self {
                if Self::contains(value) {
                    Self(value)
                }
                else {
                    panic!("expected value in range [{}...{}], got {}",
                        Self::FIRST, Self::LAST, value);
                }
            }

            fn value(&self) -> i32 { self.0 }

            fn contains(value: i32) -> bool {
                value >= Self::FIRST && value <= Self::LAST
            }

            fn random() -> Self {
                let mut rng = rand::thread_rng();
                Self::new(rng.gen_range(Self::FIRST..=Self::LAST))
            }
        }

        impl Default for $typ {
            fn default() -> Self {
                if Self::contains(Self::DEFAULT) {
                    Self::new(Self::DEFAULT)
                } else {
                    panic!("default value {} not in range [{}...{}]",
                        Self::DEFAULT, Self::FIRST, Self::LAST);
                }
            }
        }

        impl fmt::Display for $typ {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct Grade(i32);
    ranged_impl!(Grade, 0, 5, 0);

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct BadGrade(i32);
    ranged_impl!(BadGrade, 0, 5, 10);  // note: default value is outside the range!

    #[test]
    #[should_panic]
    fn test_ranged_invalid_panic() {
        let _ = Grade::new(6);  // value is outside the range
    }

    #[test]
    #[should_panic]
    fn test_ranged_invalid_default_panic() {
        let _: BadGrade = Default::default();  // default is outside the range
    }

    #[test]
    fn test_ranged_valid() {
        let grade = Grade::new(5);
        assert_eq!(grade.value(), 5);
    }

    #[test]
    fn test_ranged_contains() {
        assert!(Grade::contains(3));
    }
}
