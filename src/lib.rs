mod dx7;

/// Parsing and generating MIDI System Exclusive data.
pub trait SystemExclusiveData {
    fn from_bytes(data: Vec<u8>) -> Self;
    fn from_packed_bytes(data: Vec<u8>) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
    fn to_packed_bytes(&self) -> Vec<u8> { vec![] }
    fn data_size(&self) -> usize { 0 }
}
