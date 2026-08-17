use std::fmt;
use bit::BitIndex;
use rand::Rng;

use dbg_hex::dbg_hex;

use syxpack::{
    ParseError,
    Ranged,
    ranged_impl,
    Encoding,
    SystemExclusiveData,
    parse_or_default,
};

use crate::dx7::{
    Depth,
    Level,
    Detune,
    Sensitivity,
    Coarse,
};

use crate::dx7::envelope::Envelope;


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

impl Into<u8> for ScalingCurve {
    fn into(self) -> u8 {
        match self {
            ScalingCurve { style: CurveStyle::Linear, sign: CurveSign::Positive } => 3,
            ScalingCurve { style: CurveStyle::Linear, sign: CurveSign::Negative } => 0,
            ScalingCurve { style: CurveStyle::Exponential, sign: CurveSign::Positive } => 2,
            ScalingCurve { style: CurveStyle::Exponential, sign: CurveSign::Negative } => 1,
        }
    }
}

/// Key
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Key(i32);
ranged_impl!(Key, 0, 99, 39);  // note the default!

impl Encoding for Key { }  // identity transformation

impl Key {
    pub fn name(&self) -> String {
        let notes = [ "C", "C#", "D", "Eb", "E", "F", "F#", "G", "G#", "A", "Bb", "B" ];
        let octave: usize = self.value() as usize / 12 + 1;
        let name = notes[(self.value() % 12) as usize];
        format!("{}{}", name, octave)
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
    fn parse(data: &[u8]) -> Result<Self, ParseError> {
        Ok(Self {
            breakpoint: parse_or_default::<Key>(data[0]),
            left: Scaling { 
                depth: parse_or_default::<Level>(data[1]), 
                curve: ScalingCurve::from(data[3])
            },
            right: Scaling { 
                depth: parse_or_default::<Level>(data[2]), 
                curve: ScalingCurve::from(data[4])
            },
        })
    }

    /// Gets the SysEx bytes representing this set of parameters.
    fn to_bytes(&self) -> Vec<u8> {
        vec![
            self.breakpoint.encode(),
            self.left.depth.encode(),
            self.right.depth.encode(),
            self.left.curve.into(),
            self.right.curve.into(),
        ]
    }

    fn data_size() -> usize { 5 }
}

/// Operator mode.
#[derive(Debug, Copy, Clone)]
pub enum OperatorMode {
    Ratio,
    Fixed,
}

impl fmt::Display for OperatorMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            OperatorMode::Ratio => "ratio",
            OperatorMode::Fixed => "fixed",
        };
        write!(f, "{}", printable)
    }
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
    pub fn random() -> Self {
        Operator {
            eg: Envelope::random(),
            kbd_level_scaling: KeyboardLevelScaling::new(),
            kbd_rate_scaling: Depth::new(0),
            amp_mod_sens: Sensitivity::new(0),
            key_vel_sens: Depth::new(0),
            output_level: Level::random(),
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

        // KLS
        result.push(data[8]);  // BP
        result.push(data[9]);  // LD
        result.push(data[10]); // RD

        result.push(data[11].bit_range(0..2));  // LC
        result.push(data[11].bit_range(2..4));  // RC

        result.push(data[12].bit_range(0..3));  // RS
        result.push(data[13].bit_range(0..2));  // AMS
        result.push(data[13].bit_range(2..5));  // KVS

        result.push(data[14]);  // output level
        result.push(if data[15].bit(0) { 1 } else { 0 });  // osc mode
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
    fn parse(data: &[u8]) -> Result<Self, ParseError> {
        //dbg!(&data[0..8]);
        let eg = Envelope::parse(&data[0..8])?;
        //println!("EG = {}", eg);

        //dbg!(&data[8..13]);
        let kbd_level_scaling = KeyboardLevelScaling::parse(&data[8..13])?;
        //println!("KLS = {}", kbd_level_scaling);

        //dbg!(data[13]);
        let kbd_rate_scaling = parse_or_default::<Depth>(data[13]);
        //dbg!(kbd_rate_scaling);

        //dbg!(data[14]);
        let amp_mod_sens = parse_or_default::<Sensitivity>(data[14]);
        //dbg!(amp_mod_sens);

        let key_vel_sens = parse_or_default::<Depth>(data[15]);
        let output_level = parse_or_default::<Level>(data[16]);
        let mode = if data[17] == 0b1 { OperatorMode::Fixed } else { OperatorMode::Ratio };
        let coarse = parse_or_default::<Coarse>(data[18]);
        let fine = parse_or_default::<Level>(data[19]);

        //dbg!(data[20]);
        let detune = parse_or_default::<Detune>(data[20]);

        Ok(Self {
            eg,
            kbd_level_scaling,
            kbd_rate_scaling,
            amp_mod_sens,
            key_vel_sens,
            output_level,
            mode,
            coarse,
            fine,
            detune,
        })
    }

    /// Gets the SysEx bytes representing the operator.
    fn to_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();
        data.extend(self.eg.to_bytes());
        data.extend(self.kbd_level_scaling.to_bytes());
        data.push(self.kbd_rate_scaling.encode());
        data.push(self.amp_mod_sens.encode());
        data.push(self.key_vel_sens.encode());
        data.push(self.output_level.encode());
        data.push(self.mode as u8);
        data.push(self.coarse.encode());
        data.push(self.fine.encode());
        data.push(self.detune.encode()); // 0 = detune -7, 7 = 0, 14 = +7

        assert_eq!(data.len(), 21);

        data
    }

    fn data_size() -> usize {
        21
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EG: {}
Kbd level scaling: {}, Kbd rate scaling: {}
Amp mod sens = {}, Key vel sens = {}
Level = {}, Mode = {:?}
Coarse = {}, Fine = {}, Detune = {}",
            self.eg,
            self.kbd_level_scaling,
            self.kbd_rate_scaling,
            self.amp_mod_sens,
            self.key_vel_sens,
            self.output_level,
            self.mode,
            self.coarse,
            self.fine,
            self.detune)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::dx7::compare_slices;
    use crate::dx7::envelope::Rate;

    #[test]
    fn test_from_packed_bytes() {
        let all_data = include_bytes!("rom1a_payload.dat");

        // The first voice in ROM1A ("BRASS 1") starts at offset 4,
        // after the SysEx header.
        let voice_data = &all_data[4..];

        // The data for its OP6 is first. It is 21 bytes when packed.
        let packed_op_data = &voice_data[..21];

        let op_data = Operator::unpack(packed_op_data);

        // OP6 rates: 31 63 1c 44
        // OP6 levels: 62 62 5b 00

        // KLS: 27 36 32 05
        // - breakpoint = 27H = 39 = C3
        // - left depth = 36H = 54
        // - right depth = 32H = 50
        // - left curve and right curve packed = 05H
        // Arturia DX7 shows: breakpoint=C3, both curves=-EXP,
        // left depth = 54, right depth = 50

        // Byte #12: osc detune and rate scaling 3CH = 0111_100B
        // Detune = 0111B = 7
        // rate scaling = 100B = 4

        _ = Operator::parse(&op_data).expect("valid operator");
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
                left: Scaling { 
                    depth: Level::new(54), 
                    curve: ScalingCurve::exp_neg() 
                },
                right: Scaling { 
                    depth: Level::new(50), 
                    curve: ScalingCurve::exp_neg() 
                },
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

        let diff_offset = compare_slices(&expected_data, &data);
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
