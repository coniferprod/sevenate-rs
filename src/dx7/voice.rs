use std::fmt;

use bit::BitIndex;
use dbg_hex::dbg_hex;
use rand::seq::SliceRandom;

use crate::{
    ParseError,
    Ranged,
};

use crate::dx7::{
    Algorithm,
    Depth,
    Transpose,
    Level,
    compare_slices,
};

use crate::dx7::sysex::SystemExclusiveData;
use crate::dx7::operator::Operator;
use crate::dx7::lfo::Lfo;
use crate::dx7::envelope::Envelope;

pub const OPERATOR_COUNT: usize = 6;
pub const VOICE_PACKED_SIZE: usize = 128;
pub const VOICE_SIZE: usize = 155;

/// Voice name.
#[derive(Debug, Clone)]
pub struct VoiceName {
    value: String,
}

impl VoiceName {
    pub fn new(name: &str) -> Self {
        VoiceName { value: String::from(name) }
    }

    pub fn from_string(name: String) -> Self {
        VoiceName { value: name.clone() }
    }

    pub fn random() -> Self {
        // Five two-character syllables
        VoiceName { value: Self::make_random_phrase(5) }
    }

    // Makes a random syllable from a consonant and a vowel.
    // The result is not linguistically correct.
    fn make_random_syllable() -> String {
        // Japanese vowels and consonants. See https://www.lingvozone.com/Japanese.
        let consonants = vec![
            'k', 's', 't', 'n', 'h', 'm', 'y', 'r',
            'w', 'g', 'z', 'd', 'b', 'p'
        ];
        let vowels = vec!['a', 'i', 'u', 'e', 'o'];
        let mut rng = rand::thread_rng();
        let consonant = consonants.choose(&mut rng).unwrap();
        let vowel = vowels.choose(&mut rng).unwrap();
        format!("{}{}", consonant, vowel)
    }

    // Makes a random phrase out of `syllable_count` syllables.
    // The result is not linguistically correct.
    fn make_random_phrase(syllable_count: u32) -> String {
        let mut result = String::new();
        for _ in 1..=syllable_count {
            result.push_str(&Self::make_random_syllable());
        }
        result
    }

    pub fn value(&self) -> String {
        self.value.to_uppercase().clone()
    }
}

impl fmt::Display for VoiceName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}

/// A DX7 voice.
#[derive(Debug, Clone)]
pub struct Voice {
    pub operators: [Operator; OPERATOR_COUNT],
    pub peg: Envelope,  // pitch env
    pub alg: Algorithm,  // 1...32
    pub feedback: Depth,
    pub osc_sync: bool,
    pub lfo: Lfo,
    pub pitch_mod_sens: Depth,  // pitch mode sensitivity 0 ~7 (for all operators)
    pub transpose: Transpose,  // number of octaves to transpose (-2...+2) (12 = C2 (value is 0~48 in SysEx))
    pub name: VoiceName,
}

impl Voice {
    /// Creates a new voice and initializes it with the DX7 voice defaults.
    pub fn new() -> Self {
        Self {
            operators: [
                Operator { output_level: Level::new(0), ..Operator::new() },
                Operator { output_level: Level::new(0), ..Operator::new() },
                Operator { output_level: Level::new(0), ..Operator::new() },
                Operator { output_level: Level::new(0), ..Operator::new() },
                Operator { output_level: Level::new(0), ..Operator::new() },
                Operator { output_level: Level::new(0), ..Operator::new() },
            ],
            peg: Envelope {
                levels: [Level::new(50), Level::new(50), Level::new(50), Level::new(50)],
                ..Envelope::new()
            },
            alg: Algorithm::new(1),
            feedback: Depth::new(0),
            osc_sync: true,
            lfo: Lfo::new(),
            pitch_mod_sens: Depth::new(0),
            transpose: Transpose::new(0),
            name: VoiceName::new("INIT VOICE"),
        }
    }

    /// Pack the voice data to use in a cartridge.
    pub fn pack(data: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        let mut offset = 0;

        // The operator data is already in reverse order (OP6 first),
        // so just take each chunk and pack it.
        for _i in 0..6 {
            let op_data = &data[offset .. offset + 21];
            let op_data_packed = Operator::pack(&op_data);
            result.extend(op_data_packed);
            offset += 21;
        }

        // Copy the pitch EG as is.
        result.extend(&data[offset .. offset + 8]);
        offset += 8;

        result.push(data[offset]);  // algorithm
        offset += 1;

        let byte111 = data[offset] // feedback
            | (data[offset + 1] << 3);  // osc sync
        result.push(byte111);
        offset += 2;

        // LFO speed, delay, PMD, AMD
        result.extend(&data[offset .. offset + 4]);
        offset += 4;

        let mut byte116: u8 = data[offset];  // LFO sync
        byte116.set_bit_range(1..4, data[offset + 1]);  // LFO waveform
        byte116.set_bit_range(4..7, data[offset + 2]);  // pitch mod sens (voice)
        result.push(byte116);
        offset += 3;

        result.push(data[offset]);  // transpose
        offset += 1;

        // voice name
        result.extend(&data[offset .. offset + 10]);

        assert_eq!(result.len(), VOICE_PACKED_SIZE);
        result
    }

    /// Unpack voice data from a cartridge.
    /// Returns a vector to use for normal parsing.
    pub fn unpack(data: &[u8]) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();

        let mut offset = 0;
        for i in (0..6).rev() {  // NOTE: reverse order!
            let size = 17;  // packed operator data length
            result.extend(Operator::unpack(&data[offset .. offset + size]));
            offset += size;
        }

        // Now offset should be at the start of the pitch EG.
        assert_eq!(offset, 102);

        result.extend(&data[offset .. offset + 8]);  // PEG = 4xrate + 4xlevel
        offset += 8;

        // Algorithm
        assert_eq!(offset, 110);
        let alg = data[offset];
        //dbg_hex!(alg);
        result.push(alg);
        offset += 1;

        let feedback = data[offset].bit_range(0..3);
        //dbg_hex!(feedback);
        result.push(feedback); // feedback
        result.push(if data[offset].bit(3) { 1 } else { 0 }); // osc sync
        offset += 1;

        result.extend(Lfo::unpack(&data[offset .. offset + 5]));
        offset += 4;  // we'll use the last byte soon
        result.push(data[offset].bit_range(4..6));  // pitch mod sens
        offset += 1;

        result.push(data[offset]); // transpose
        offset += 1;

        // Voice name (last 10 characters)
        result.extend(&data[offset .. offset + 10]);
        offset += 10;

        assert_eq!(offset, VOICE_PACKED_SIZE);
        assert_eq!(result.len(), VOICE_SIZE);

        result
    }
}

impl Default for Voice {
    fn default() -> Voice {
        Voice::new()
    }
}

impl SystemExclusiveData for Voice {
    fn parse(data: &[u8]) -> Result<Voice, ParseError> {
        //dbg_hex!(data);

        // Note that the operator data is in reverse order:
        // OP6 is first, OP1 is last.

        //dbg!(&data[0..21]);
        let op6 = Operator::parse(&data[0..21])?;

        //dbg!(&data[21..42]);
        let op5 = Operator::parse(&data[21..42])?;

        //dbg!(&data[42..63]);
        let op4 = Operator::parse(&data[42..63])?;

        //dbg!(&data[63..84]);
        let op3 = Operator::parse(&data[63..84])?;

        //dbg!(&data[84..105]);
        let op2 = Operator::parse(&data[84..105])?;

        //dbg!(&data[105..126]);
        let op1 = Operator::parse(&data[105..126])?;

        //dbg!(&data[126..134]);
        let peg = Envelope::parse(&data[126..134])?;

        //dbg_hex!(data[134]);
        let alg = Algorithm::new(data[134] as i32 + 1);  // 0...31 to 1...32

        //dbg_hex!(data[135]);
        let feedback = Depth::new(data[135].into());

        //dbg_hex!(data[144]);
        //let transpose = Transpose::from(data[144]);
        let transpose = Transpose::new(data[144] as i32 - 24);

        //dbg_hex!(&data[145..155]);
        let name = VoiceName::from_string(String::from_utf8(data[145..155].to_vec()).unwrap());

        Ok(Voice {
            operators: [
                op1,
                op2,
                op3,
                op4,
                op5,
                op6,
            ],
            peg,
            alg,
            feedback,
            osc_sync: data[136] == 1,
            lfo: Lfo::parse(&data[137..143])?,
            pitch_mod_sens: Depth::new(data[143].into()),
            transpose,
            name,
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        for i in (0..6).rev() {  // NOTE: reverse order!
            data.extend(self.operators[i].to_bytes());
        }

        data.extend(self.peg.to_bytes());

        data.push(self.alg.as_byte());
        data.push(self.feedback.as_byte());
        data.push(if self.osc_sync { 1 } else { 0 });
        data.extend(self.lfo.to_bytes());
        data.push(self.pitch_mod_sens.as_byte());
        data.push(self.transpose.as_byte());

        let padded_name = format!("{:<10}", self.name.value());
        data.extend(padded_name.into_bytes());

        assert_eq!(data.len(), Self::DATA_SIZE);

        data
    }

    const DATA_SIZE: usize = 155;
}

impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}
OP1: {}
OP2: {}
OP3: {}
OP4: {}
OP5: {}
OP6: {}
PEG: {}
ALG: {}, feedback = {}, osc sync = {}
LFO: {}
Transpose: {}",
            self.name,
            self.operators[0],
            self.operators[1],
            self.operators[2],
            self.operators[3],
            self.operators[4],
            self.operators[5],
            self.peg,
            self.alg,
            self.feedback,
            self.osc_sync,
            self.lfo,
            self.transpose)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::dx7::*;
    use crate::dx7::operator::*;
    use crate::dx7::lfo::*;
    use crate::dx7::envelope::*;

    /// Makes a new voice based on the "BRASS1" settings in the DX7 manual.
    pub fn make_brass1() -> Voice {
        let kbd_level_scaling = KeyboardLevelScaling {
            breakpoint: Key::new(60 - 21),
            left: Scaling { depth: Level::new(0), curve: ScalingCurve::lin_pos() },
            right: Scaling { depth: Level::new(0), curve: ScalingCurve::lin_pos() },
        };

        // Make one operator and then specify the differences to the others.
        let op = Operator {
            key_vel_sens: Depth::new(2),
            ..Operator::new()
        };

        let op6 = Operator {
            eg: Envelope::new_rate_level(
                [Rate::new(49), Rate::new(99), Rate::new(28), Rate::new(68)],
                [Level::new(98), Level::new(98), Level::new(91), Level::new(0)]),
            kbd_level_scaling: KeyboardLevelScaling {
                left: Scaling { depth: Level::new(54), curve: ScalingCurve::exp_neg() },
                right: Scaling { depth: Level::new(50), curve: ScalingCurve::exp_neg() },
                ..kbd_level_scaling
            },
            kbd_rate_scaling: Depth::new(4),
            output_level: Level::new(82),
            ..op
        };

        let op5 = Operator {
            eg: Envelope::new_rate_level(
                [Rate::new(77), Rate::new(36), Rate::new(41), Rate::new(71)],
                [Level::new(99), Level::new(98), Level::new(98), Level::new(0)]),
            kbd_level_scaling,
            output_level: Level::new(98),
            detune: Detune::new(1),
            ..op
        };

        let op4 = Operator {
            eg: op5.eg.clone(),
            kbd_level_scaling,
            output_level: Level::new(99),
            ..op
        };

        let op3 = Operator {
            eg: Envelope::new_rate_level(
                [Rate::new(77), Rate::new(76), Rate::new(82), Rate::new(71)],
                [Level::new(99), Level::new(98), Level::new(98), Level::new(0)]),
            kbd_level_scaling,
            output_level: Level::new(99),
            detune: Detune::new(-2),
            ..op
        };

        let op2 = Operator {
            eg: Envelope::new_rate_level(
                [Rate::new(62), Rate::new(51), Rate::new(29), Rate::new(71)],
                [Level::new(82), Level::new(95), Level::new(96), Level::new(0)]),
            kbd_level_scaling: KeyboardLevelScaling {
                breakpoint: Key::new(48 - 21),
                left: Scaling { depth: Level::new(0), curve: ScalingCurve::lin_pos() },
                right: Scaling { depth: Level::new(7), curve: ScalingCurve::exp_neg() },
            },
            key_vel_sens: Depth::new(0),
            output_level: Level::new(86),
            coarse: Coarse::default(),
            detune: Detune::new(7),
            ..op
        };

        let op1 = Operator {
            eg: Envelope::new_rate_level(
                [Rate::new(72), Rate::new(76), Rate::new(99), Rate::new(71)],
                [Level::new(99), Level::new(88), Level::new(96), Level::new(0)]),
            kbd_level_scaling: KeyboardLevelScaling {
                right: Scaling { depth: Level::new(14), curve: ScalingCurve::lin_pos() },
                ..kbd_level_scaling
            },
            key_vel_sens: Depth::new(0),
            output_level: Level::new(98),
            coarse: Coarse::default(),
            detune: Detune::new(7),
            ..op
        };

        Voice {
            operators: [op1, op2, op3, op4, op5, op6],
            peg: Envelope::new_rate_level(
                [Rate::new(84), Rate::new(95), Rate::new(95), Rate::new(60)],
                [Level::new(50), Level::new(50), Level::new(50), Level::new(50)]),
            alg: Algorithm::new(22),
            feedback: Depth::new(7),
            osc_sync: true,
            lfo: Lfo {
                speed: Level::new(37),
                delay: Level::new(0),
                pmd: Level::new(5),
                amd: Level::new(0),
                sync: false,
                waveform: LfoWaveform::Sine,
            },
            pitch_mod_sens: Depth::new(3),
            transpose: Transpose::new(0),
            name: VoiceName::new("BRASS   1 "),
        }
    }

    #[test]
    fn test_voice_packed_length() {
        let brass1 = make_brass1();
        let voice_data = brass1.to_bytes();
        let packed_voice_data = Voice::pack(&voice_data);
        assert_eq!(packed_voice_data.len(), 128);
    }

    #[test]
    fn test_voice_to_packed_bytes() {
        let data = include_bytes!("rom1a_payload.dat");

        // The first voice in ROM1A ("BRASS 1") starts at offset 4,
        // after the SysEx header. It is 128 bytes when packed.
        let voice_data = &data[4..132];
        let brass1 = make_brass1();
        let brass1_data = Voice::pack(&brass1.to_bytes());

        let diff_offset = compare_slices(voice_data, &brass1_data);
        match diff_offset {
            Some(offset) => {
                println!("Vectors differ at offset {:?}", offset);
                println!("Expected = {}, actual = {}", voice_data[offset], brass1_data[offset]);
            },
            None => println!("Vectors are the same")
        }

        assert_eq!(voice_data, brass1_data);
    }
}
