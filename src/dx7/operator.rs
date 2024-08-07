use std::fmt;
use bit::BitIndex;
use rand::Rng;

use crate::ParseError;

use crate::dx7::{
    Ranged,
    Depth,
    Level,
    Detune,
    Sensitivity,
    Coarse,
    first_different_offset
};

use crate::dx7::envelope::{
    Envelope,
    Rate
};

use crate::dx7::sysex::SystemExclusiveData;

/// Scaling curve style.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CurveStyle {
    Linear,
    Exponential
}

impl fmt::Display for CurveStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CurveStyle::Linear => write!(f, "LIN"),
            CurveStyle::Exponential => write!(f, "EXP"),
        }
    }
}

/// Scaling curve sign.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CurveSign {
    Negative,
    Positive,
}

impl fmt::Display for CurveSign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if *self == CurveSign::Positive { "+" } else { "-" })
    }
}

/// Scaling curve settings.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ScalingCurve {
    pub style: CurveStyle,
    pub sign: CurveSign,
}

impl ScalingCurve {
    /// Makes a linear positive scaling curve.
    pub fn lin_pos() -> Self {
        ScalingCurve { style: CurveStyle::Linear, sign: CurveSign::Positive }
    }

    /// Makes a linear negative scaling curve.
    pub fn lin_neg() -> Self {
        ScalingCurve { style: CurveStyle::Linear, sign: CurveSign::Negative }
    }

    /// Makes an exponential positive scaling curve.
    pub fn exp_pos() -> Self {
        ScalingCurve { style: CurveStyle::Exponential, sign: CurveSign::Positive }
    }

    /// Makes an exponential negative scaling curve.
    pub fn exp_neg() -> Self {
        ScalingCurve { style: CurveStyle::Exponential, sign: CurveSign::Negative }
    }

    /// Gets the SysEx byte for this scaling curve.
    pub fn as_byte(&self) -> u8 {
        match self {
            ScalingCurve { style: CurveStyle::Linear, sign: CurveSign::Positive } => 3,
            ScalingCurve { style: CurveStyle::Linear, sign: CurveSign::Negative } => 0,
            ScalingCurve { style: CurveStyle::Exponential, sign: CurveSign::Positive } => 2,
            ScalingCurve { style: CurveStyle::Exponential, sign: CurveSign::Negative } => 1,
        }
    }
}

impl fmt::Display for ScalingCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.sign, self.style)
    }
}

impl From<u8> for ScalingCurve {
    fn from(item: u8) -> Self {
        match item {
            0 => ScalingCurve::lin_neg(),
            1 => ScalingCurve::exp_neg(),
            2 => ScalingCurve::exp_pos(),
            3 => ScalingCurve::lin_pos(),
            _ => panic!("expected value in range 0...3, got {}", item)
        }
    }
}

/// Key (MIDI note).
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Key {
    value: i32
}

impl Key {
    pub fn as_byte(&self) -> u8 {
        self.value as u8
    }

    pub fn name(&self) -> String {
        let notes = [ "C", "C#", "D", "Eb", "E", "F", "F#", "G", "G#", "A", "Bb", "B" ];
        let octave: usize = self.value as usize / 12 + 1;
        let name = notes[(self.value % 12) as usize];
        format!("{}{}", name, octave)
    }
}

impl Default for Key {
    fn default() -> Key {
        Key::new(39)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<u8> for Key {
    fn from(item: u8) -> Self {
        Key::new(item as i32)
    }
}

impl Ranged for Key {
    fn new(value: i32) -> Self {
        if Key::is_valid(value) {
            Self { value }
        }
        else {
            panic!("expected value in range {}...{}, got {}",
                Self::first(), Self::last(), value);
        }
    }

    fn value(&self) -> i32 { self.value }

    fn first() -> i32 { 0 }
    fn last() -> i32 { 99 }

    fn is_valid(value: i32) -> bool {
        value >= Self::first() && value <= Self::last()
    }

    fn random_value() -> Self {
        let mut rng = rand::thread_rng();
        Key::new(rng.gen_range(Self::first()..=Self::last()))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Scaling {
    pub depth: Level,
    pub curve: ScalingCurve,
}

/// Keyboard level scaling.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct KeyboardLevelScaling {
    pub breakpoint: Key, // 0 ~ 99 (A-1 ~ C8)
    pub left: Scaling,
    pub right: Scaling,
}

impl KeyboardLevelScaling {
    /// Creates new keyboard level scaling settings with DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            breakpoint: Key::default(),  // Yamaha C3 is 60 - 21 = 39
            left: Scaling { depth: Level::new(0), curve: ScalingCurve::lin_neg() },
            right: Scaling { depth: Level::new(0), curve: ScalingCurve::lin_neg() } // is it?
        }
    }
}

impl fmt::Display for KeyboardLevelScaling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "breakpoint = {}, left depth = {}, right depth = {}, left curve = {}, right curve = {}",
            self.breakpoint, self.left.depth, self.right.depth, self.left.curve, self.right.curve)
    }
}

impl SystemExclusiveData for KeyboardLevelScaling {
    /// Makes new keyboard level scaling settings from SysEx bytes.
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        Ok(Self {
            breakpoint: Key::new(data[0].into()),
            left: Scaling { depth: Level::new(data[1].into()), curve: ScalingCurve::from(data[3]) },
            right: Scaling { depth: Level::new(data[2].into()), curve: ScalingCurve::from(data[4]) },
        })
    }

    /// Gets the SysEx bytes representing this set of parameters.
    fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.breakpoint.as_byte(),
            self.left.depth.as_byte(),
            self.right.depth.as_byte(),
            self.left.curve.as_byte(),
            self.right.curve.as_byte(),
        ]
    }

    fn data_size(&self) -> usize { 5 }
}

/// Operator mode.
#[derive(Debug, Copy, Clone)]
pub enum OperatorMode {
    Ratio,
    Fixed,
}

/// Operator.
#[derive(Debug, Clone, Copy)]
pub struct Operator {
    pub eg: Envelope,
    pub kbd_level_scaling: KeyboardLevelScaling,
    pub kbd_rate_scaling: Depth, // 0 ~ 7
    pub amp_mod_sens: Sensitivity,  // 0 ~ 3
    pub key_vel_sens: Depth,  // 0 ~ 7
    pub output_level: Level,
    pub mode: OperatorMode,
    pub coarse: Coarse,  // 0 ~ 31
    pub fine: Level,  // 0 ~ 99
    pub detune: Detune,   // -7 ~ 7
}

impl Operator {
    /// Creates a new operator and initializes it with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            eg: Envelope::new(),
            kbd_level_scaling: KeyboardLevelScaling::new(),
            kbd_rate_scaling: Depth::new(0),
            amp_mod_sens: Sensitivity::new(0),
            key_vel_sens: Depth::new(0),
            output_level: Level::new(0),
            mode: OperatorMode::Ratio,
            coarse: Coarse::new(1),
            fine: Level::new(0),  // TODO: voice init for fine is "1.00 for all operators", should this be 0 or 1?
            detune: Detune::new(0),
        }
    }

    /// Makes a new random operator.
    pub fn new_random() -> Self {
        Operator {
            eg: Envelope::new_random(),
            kbd_level_scaling: KeyboardLevelScaling::new(),
            kbd_rate_scaling: Depth::new(0),
            amp_mod_sens: Sensitivity::new(0),
            key_vel_sens: Depth::new(0),
            output_level: Level::random_value(),
            mode: OperatorMode::Ratio,
            coarse: Coarse::new(1),
            fine: Level::new(0),
            detune: Detune::new(0),
        }
    }

    /// Unpacks operator data from a cartridge.
    /// Returns the data in the same format as for a single voice.
    pub fn unpack(data: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        // EG data is unpacked
        result.extend(data[0..8].to_vec());  

        result.push(data[8]);  // BP
        result.push(data[9]);  // LD
        result.push(data[10]); // RD

        result.push(data[11] >> 4);   // LC
        result.push(data[11] & 0x0f); // RC

        result.push(data[12].bit_range(0..3));  // RS
        result.push(data[13].bit_range(0..2));  // AMS
        result.push(data[13].bit_range(2..5));  // KVS

        result.push(data[14]);  // output level
        result.push(if data[15].bit(0) { 1 } else { 0 });  // op mode
        result.push(data[15].bit_range(1..6)); // coarse
        result.push(data[16]); // fine
        result.push(data[12].bit_range(3..7)); // detune

        result
    }

    /// Packs the operator bytes for use in a voice inside a cartridge.
    pub fn pack(data: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        // Copy the EG bytes as is.
        result.extend(&data[0 .. 8]);

        // KLS breakpoint, left and right depths:
        result.push(data[8]);
        result.push(data[9]);
        result.push(data[10]);

        // Combine bytes 11 and 12 into one:
        result.push(data[11] | (data[12] << 2));

        result.push(data[13] | (data[20] << 3));

        result.push(data[14] | (data[15] << 2));
        result.push(data[16]);

        result.push(data[17] | (data[18] << 1));  // coarse + mode
        result.push(data[19]);  // fine

        assert_eq!(result.len(), 17);

        result
    }
}

impl SystemExclusiveData for Operator {
    /// Makes a new operator from SysEx bytes.
    fn from_bytes(data: &[u8]) -> Result<Self, ParseError> {
        Ok(Self {
            eg: Envelope::from_bytes(&data[0..8])?,
            kbd_level_scaling: KeyboardLevelScaling::from_bytes(&data[8..13])?,
            kbd_rate_scaling: Depth::new(data[13].into()),
            amp_mod_sens: Sensitivity::new(data[14].into()),
            key_vel_sens: Depth::new(data[15].into()),
            output_level: Level::new(data[16].into()),
            mode: if data[17] == 0b1 { OperatorMode::Fixed } else { OperatorMode::Ratio },
            coarse: Coarse::new(data[18].into()),
            fine: Level::new(data[19].into()),
            detune: Detune::from(data[20]),
        })
    }

    /// Gets the SysEx bytes representing the operator.
    fn to_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.extend(self.eg.to_bytes());
        data.extend(self.kbd_level_scaling.to_bytes());
        data.push(self.kbd_rate_scaling.as_byte());
        data.push(self.amp_mod_sens.as_byte());
        data.push(self.key_vel_sens.as_byte());
        data.push(self.output_level.as_byte());
        data.push(self.mode as u8);
        data.push(self.coarse.as_byte());
        data.push(self.fine.as_byte());
        data.push(self.detune.as_byte()); // 0 = detune -7, 7 = 0, 14 = +7

        assert_eq!(data.len(), 21);

        data
    }

    fn data_size(&self) -> usize { 21 }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EG: {}
Kbd level scaling: {}, Kbd rate scaling: {}
Amp mod sens = {}, Key vel sens = {}
Level = {}, Mode = {:?}
Coarse = {}, Fine = {}, Detune = {}
",
            self.eg,
            self.kbd_level_scaling,
            self.kbd_rate_scaling.value,
            self.amp_mod_sens,
            self.key_vel_sens.value,
            self.output_level.value,
            self.mode,
            self.coarse.value,
            self.fine.value,
            self.detune.value)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_from_bytes() {
        let data = vec![
            0x03u8, 0x47, 0x00, 0x03, 0x00, 0x07, 0x63, 0x23,  // rate and level
            0x63, 0x57, 0x63, 0x63, 0x63,  // kbd level scaling
            0x00, 0x00, 0x00,
            0x11,  // output level
            0x00,   // osc mode
            0x00, 0x00, 0x00, // coarse, fine, detune
        ];
        assert_eq!(data.len(), 21);
        _ = Operator::from_bytes(&data).expect("valid operator");
    }

    #[test]
    fn test_pack() {
        let op = Operator {
            eg: Envelope {
                rates: [Rate::new(49), Rate::new(99), Rate::new(28), Rate::new(68)],
                levels: [Level::new(98), Level::new(98), Level::new(91), Level::new(0)]
            },
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: Key::new(39),
                left: Scaling { depth: Level::new(54), curve: ScalingCurve::exp_neg() },
                right: Scaling { depth: Level::new(50), curve: ScalingCurve::exp_neg() },
            },
            kbd_rate_scaling: Depth::new(4),
            amp_mod_sens: Sensitivity::new(0),
            key_vel_sens: Depth::new(2),
            output_level: Level::new(82),
            mode: OperatorMode::Ratio,
            coarse: Coarse::new(1),
            fine: Level::new(0),
            detune: Detune::new(0),
        };

        let data = Operator::pack(&op.to_bytes());

        let expected_data = vec![
            0x31u8, 0x63, 0x1c, 0x44, 0x62, 0x62, 0x5b, 0x00,
            0x27, 0x36, 0x32, 0x05, 0x3c, 0x08, 0x52, 0x02, 0x00];

        let diff_offset = first_different_offset(&expected_data, &data);
        match diff_offset {
            Some(offset) => {
                println!("Vectors differ at offset {:?}", offset);
                println!("Expected = {}, actual = {}", expected_data[offset], data[offset]);
            },
            None => println!("Vectors are the same")
        }

        assert_eq!(data, expected_data);
    }
}
