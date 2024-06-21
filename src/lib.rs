mod dx7;

/// Parsing and generating MIDI System Exclusive data.
pub trait SystemExclusiveData {
    fn from_bytes(data: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn data_size(&self) -> usize;
}

/// An `i32`` constrained to a range.
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
