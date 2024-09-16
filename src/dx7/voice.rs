use std::fmt;
use bit::BitIndex;

use dbg_hex::dbg_hex;

use crate::{
    ParseError,
    Ranged,
};

use crate::dx7::{
    Algorithm,
    Depth,
    Transpose,
    Level,
    first_different_offset,
};

use crate::dx7::sysex::SystemExclusiveData;
use crate::dx7::operator::Operator;
use crate::dx7::lfo::Lfo;
use crate::dx7::envelope::Envelope;

const OPERATOR_COUNT: usize = 6;

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
    pub name: String,
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
            name: "INIT VOICE".to_string(),
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

        assert_eq!(offset, 128);

        result
    }
}

impl Default for Voice {
    fn default() -> Voice {
        Voice::new()
    }
}

impl SystemExclusiveData for Voice {
    fn from_bytes(data: &[u8]) -> Result<Voice, ParseError> {
        //dbg_hex!(data);

        // Note that the operator data is in reverse order:
        // OP6 is first, OP1 is last.

        //dbg!(&data[0..21]);
        let op6 = Operator::from_bytes(&data[0..21])?;

        //dbg!(&data[21..42]);
        let op5 = Operator::from_bytes(&data[21..42])?;

        //dbg!(&data[42..63]);
        let op4 = Operator::from_bytes(&data[42..63])?;

        //dbg!(&data[63..84]);
        let op3 = Operator::from_bytes(&data[63..84])?;

        //dbg!(&data[84..105]);
        let op2 = Operator::from_bytes(&data[84..105])?;

        //dbg!(&data[105..126]);
        let op1 = Operator::from_bytes(&data[105..126])?;

        //dbg!(&data[126..134]);
        let peg = Envelope::from_bytes(&data[126..134])?;

        //dbg_hex!(data[134]);
        let alg = Algorithm::new(data[134] as i32 + 1);  // 0...31 to 1...32

        //dbg_hex!(data[135]);
        let feedback = Depth::new(data[135].into());

        //dbg_hex!(data[144]);
        //let transpose = Transpose::from(data[144]);
        let transpose = Transpose::new(data[144] as i32 - 24);

        //dbg_hex!(&data[145..155]);
        let name = String::from_utf8(data[145..155].to_vec()).unwrap();

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
            lfo: Lfo::from_bytes(&data[137..143])?,
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

        let padded_name = format!("{:<10}", self.name);
        data.extend(padded_name.into_bytes());

        assert_eq!(data.len(), Self::DATA_SIZE);

        data
    }

    const DATA_SIZE: usize = 155;
}

impl fmt::Display for Voice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "==========
{}
==========
OP1: {}
OP2: {}
OP3: {}
OP4: {}
OP5: {}
OP6: {}
PEG: {}
ALG: {}, feedback = {}, osc sync = {}
LFO: {}
Transpose: {}
",
            self.name,
            self.operators[0],
            self.operators[1],
            self.operators[2],
            self.operators[3],
            self.operators[4],
            self.operators[5],
            self.peg,
            self.alg.value(),
            self.feedback.value(),
            self.osc_sync,
            self.lfo,
            self.transpose.value())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::dx7::make_brass1;

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

        let diff_offset = first_different_offset(voice_data, &brass1_data);
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
