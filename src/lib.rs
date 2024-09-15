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

// The `ranged_impl` macro generates an implementation of the `Ranged` trait,
// along with implementations of the `Default` and `Display` traits based on
// the values supplied as parameters (type name, min, max, default).
#[macro_export]
macro_rules! ranged_impl {
    ($typ:ty, $min:expr, $max:expr, $default:expr) => {
        impl Ranged for $typ {
            const MIN: i32 = $min;
            const MAX: i32 = $max;
            const DEFAULT: i32 = $default;

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

        impl Default for $typ {
            fn default() -> Self {
                Self::new(Self::DEFAULT)
            }
        }

        impl fmt::Display for $typ {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    }
}
