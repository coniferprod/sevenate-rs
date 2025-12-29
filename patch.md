# Dissection of a DX7 patch

The SysEx file for an individual voice is 163 bytes:

    * Two bytes for the SysEx initiator and manufacturer (F0 43)
    * Four bytes for the header (00 00 01 1B)
    * The actual voice payload (156 bytes)
    * One byte for the SysEx terminator (F7)

Here is the content of the payload of first voice in ROM1A, namely "BRASS  1 ".

Operator data is 6 * 21 = 126 bytes (0...125).

    0..20: operator 6
    31 63 1C 44 62 62 5B 00
    27 36 32 01 01 04 00 02
    52 00 01 00 07

    21..41: operator 5
    4D 24 29 47 63 62 62 00
    27 00 00 03 03 00 00 02
    62 00 01 00 08

    42...62: op4
    4D 24 29 47 63 62 62 00
    27 00 00 03 03 00 00 02
    63 00 01 00 07

    63..83: op3
    4D 4C 52 47 63 62 62 00
    27 00 00 03 03 00 00 02
    63 00 01 00 05

    84..104: op2
    3E 33 1D 47 52 5F 60 00
    1B 00 07 03 01 00 00 00
    56 00 00 00 0E

    105..125: op1
    48 4C 63 47 63 58 60 00
    27 00 0E 03 03 00 00 00
    62 00 00 00 0E

    PEG
    126: 54 5F 5F 3C 32 32 32 32

    134: algorithm 15H = 21 = #22
    135: feedback 07
    136: osc sync 01
    137: LFO: 25 00 05 00 00 04
    143: 03
    144: 18
    145: name = 42 52 41 53 53 20 20 20 31 20
    155: 2BH = 101011 = checksum

## Packed voice data

In a cartridge, the voice data is packed to fit into 128 bytes.

Here is the first voice in the ROM1A cartridge ("BRASS  1 "), packed:

    0x31, 0x63, 0x1c, 0x44,  // OP6 EG rates (49 99 28 68)
    0x62, 0x62, 0x5b, 0x00,  // OP6 EG levels
    0x27,                    // level scl brkpt
    0x36,                    // scl left depth
    0x32,                    // scl right depth
    0x05,                    // scl left curve and right curve
    0x3c,                    // osc detune and osc rate scale
    0x08,                    // key vel sens and amp mod sens
    0x52,                    // OP6 output level
    0x02,                    // freq coarse and osc mode
    0x00,                    // freq fine
    0x4d, 0x24, 0x29, 0x47,  // same for OP5
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x40,
    0x08,
    0x62,
    0x02,
    0x00,
    0x4d, 0x24, 0x29, 0x47,  // OP4
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x38,
    0x08,
    0x63,
    0x02,
    0x00,
    0x4d, 0x4c, 0x52, 0x47,  // OP3
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x28,
    0x08,
    0x63,
    0x02,
    0x00,
    0x3e, 0x33, 0x1d, 0x47,  // OP2
    0x52, 0x5f, 0x60, 0x00,
    0x1b,
    0x00,
    0x07,
    0x07,
    0x70,
    0x00,
    0x56,
    0x00,
    0x00,
    0x48, 0x4c, 0x63, 0x47,  // OP1
    0x63, 0x58, 0x60, 0x00,
    0x27,
    0x00,
    0x0e,
    0x0f,
    0x70,
    0x00,
    0x62,
    0x00,
    0x00,
    0x54, 0x5f, 0x5f, 0x3c,  // PEG rates
    0x32, 0x32, 0x32, 0x32,  // PEG levels
    0x15,                    // ALG
    0x0f,                    // osc key sync and feedback
    0x25,                    // LFO speed
    0x00,                    // LFO delay
    0x05,                    // LFO pitch mod dep
    0x00,                    // LFO amp mod dep
    0x38,                    // LFO pitch mode sens, wave, sync
    0x18,                    // transpose
    0x42, 0x52, 0x41, 0x53, 0x53, 0x20, 0x20, 0x20, 0x31, 0x20,   // name (10 characters)

## Patch data structure

Packed data format:

    0x31, 0x63, 0x1c, 0x44,  // OP6 EG rates
    0x62, 0x62, 0x5b, 0x00,  // OP6 EG levels
    0x27,                    // level scl brkpt
    0x36,                    // scl left depth
    0x32,                    // scl right depth
    0x05,                    // scl left curve and right curve
    0x3c,                    // osc detune and osc rate scale
    0x08,                    // key vel sens and amp mod sens
    0x52,                    // OP6 output level
    0x02,                    // freq coarse and osc mode
    0x00,                    // freq fine
    0x4d, 0x24, 0x29, 0x47,  // same for OP5
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x40,
    0x08,
    0x62,
    0x02,
    0x00,
    0x4d, 0x24, 0x29, 0x47,  // OP4
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x38,
    0x08,
    0x63,
    0x02,
    0x00,
    0x4d, 0x4c, 0x52, 0x47,  // OP3
    0x63, 0x62, 0x62, 0x00,
    0x27,
    0x00,
    0x00,
    0x0f,
    0x28,
    0x08,
    0x63,
    0x02,
    0x00,
    0x3e, 0x33, 0x1d, 0x47,  // OP2
    0x52, 0x5f, 0x60, 0x00,
    0x1b,
    0x00,
    0x07,
    0x07,
    0x70,
    0x00,
    0x56,
    0x00,
    0x00,
    0x48, 0x4c, 0x63, 0x47,  // OP1
    0x63, 0x58, 0x60, 0x00,
    0x27,
    0x00,
    0x0e,
    0x0f,
    0x70,
    0x00,
    0x62,
    0x00,
    0x00,
    0x54, 0x5f, 0x5f, 0x3c,  // PEG rates
    0x32, 0x32, 0x32, 0x32,  // PEG levels
    0x15,                    // ALG
    0x0f,                    // osc key sync and feedback
    0x25,                    // LFO speed
    0x00,                    // LFO delay
    0x05,                    // LFO pitch mod dep
    0x00,                    // LFO amp mod dep
    0x38,                    // LFO pitch mode sens, wave, sync
    0x18,                    // transpose
    0x42, 0x52, 0x41, 0x53, 0x53, 0x20, 0x20, 0x20, 0x31, 0x20,   // name (10 characters)

Note that there seems to be an error in the DX7 packed format description.
I couldn't have made this without the information found therein, but the packed LFO caused
some trouble. The document describes byte 116 of the packed format like this:

    byte             bit #
    #     6   5   4   3   2   1   0   param A       range  param B       range
    ----  --- --- --- --- --- --- ---  ------------  -----  ------------  -----
    116  |  LPMS |      LFW      |LKS| LF PT MOD SNS 0-7   WAVE 0-5,  SYNC 0-1

Actually it seems to be like this:

    byte             bit #
    #     6   5   4   3   2   1   0   param A       range  param B       range
    ----  --- --- --- --- --- --- ---  ------------  -----  ------------  -----
    116  |   LPMS    |  LFW      |LKS| LF PT MOD SNS 0-7   WAVE 0-5,  SYNC 0-1

The LFO pitch modulation sensitivity value (three bits, 0...7) is in bits 4...6,
and the LFO waveform (four bits, 0...5) is in bits 1...3.

I cross-checked this with the "BRASS 1" patch from the original ROM1 cartridge data.
The corresponding byte in the original data is 0x38 = 0b00111000, which parses to
sync = false, LFO waveform = 4 or sine, and pitch mod sens = 3. These match the
patch chart on page 28 of the DX7 Operating Manual.
