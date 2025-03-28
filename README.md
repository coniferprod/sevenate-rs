# sevenate

Rust library for working with Yamaha DX7 patches.

## Subrange types in Rust (or not)

Rust does not have subrange types like Ada. This makes it difficult
to create data types that wrap a value and enforce it to a certain
range.

The Yamaha DX7 patch data model is full of values that could be
expressed as subrange types. For example, the algorithm used in a
patch has the values 1 to 32. The detune is from -7 to 7, etc.

I have tried various ways of implementing and using subrange types
in Rust. Most attempts have resulted in a lot of boilerplate code.

### Using const generics

When [const generics](https://doc.rust-lang.org/reference/items/generics.html#const-generics) were stabilized in Rust,
I tried to implement a struct that has `const` parameters for the start
and end of the range. That ended up looking really ugly and was
cumbersome to use:

    /// A simple struct for wrapping an `i32`
    /// with const generic parameters to limit
    /// the range of allowed values.
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct RangedInteger<const MIN: i32, const MAX: i32> {
        value: i32,
    }

I wanted to make it easy to create values from specializations of
this generic struct, so I added a constructor that uses the
range defined for the values:

    impl <const MIN: i32, const MAX: i32> RangedInteger<MIN, MAX> {
        /// Makes a new ranged integer if the value is in the allowed range, otherwise panics.
        pub fn new(value: i32) -> Self {
            let range = Self::range();
            if range.contains(&value) {
                Self { value }
            }
            else {
                panic!("new() expected value in range {}...{}, got {}",
                    range.start(), range.end(), value);
            }
        }

        /// Gets the range of allowed values as an inclusive range,
        /// constructed from the generic parameters.
        pub fn range() -> RangeInclusive<i32> {
            MIN ..= MAX
        }

I also wanted an easy way to get a random value that falls in the
range of the type:

        /// Gets a random value that is in the range of allowed values.
        pub fn random_value() -> i32 {
            let mut rng = rand::thread_rng();
            let range = Self::range();
            rng.gen_range(*range.start() ..= *range.end())
        }
    }

When I want to make a new generic struct, this is what
I would have to do:

    /// Private generic type for the value stored in a `Volume`.
    type VolumeValue = RangedInteger::<0, 127>;

    /// Wrapper for volume parameter.
    #[derive(Debug, Copy, Clone)]
    pub struct Volume {
        value: VolumeValue,  // private field to prevent accidental range violations
    }

    impl Volume {
        /// Makes a new `Volume` initialized with the specified value.
        pub fn new(value: i32) -> Self {
            Self { value: VolumeValue::new(value) }
        }

        /// Gets the wrapped value.
        pub fn value(&self) -> i32 {
            self.value.value
        }
    }

There would also be other traits to implement, like `Default` and
`From<u8>`. Finally I would be able to make objects of type `Volume`:

    let volume = Volume::new(100);

When I wanted to use the value that was wrapped inside the type,
I would have to use `volume.value()`. Setting this all up
is not that far from using a newtype, but needs a lot of boilerplate.
The next obvious step would be to make a macro that generates all the
necessary boilerplate, but I didn't want to go that way, at least not yet.

### Using the `nutype` crate

Another way is to use a crate like [nutype](https://github.com/greyblake/nutype). It is a very useful crate,
but you need to use a crate-specific attribute, and starting
with [nutype 0.4](https://users.rust-lang.org/t/nutype-0-4-0-released/102889) you will need to specify your regular attributes
inside the `nutype` attribute, like this:

    /// MIDI channel
    #[nutype(
        validate(greater_or_equal = 1, less_or_equal = 16),
        derive(Debug, Copy, Clone, PartialEq, Eq)
    )]
    pub struct MIDIChannel(u8);

So, `nutype` uses the Rust [newtype pattern](https://effective-rust.com/newtype.html) and augments it with
information about the validation rules that should be applied
to values of that type.

### Using the `deranged` crate

Another possible create to use is [deranged](https://crates.io/crates/deranged).
It is labeled as proof-of-concept and very scarcely documented, but from what I can
gather, it also uses const generics.

If I were to define a new data type for a 7-bit byte, which is what is used in
MIDI message payloads, I would do it like this using the `deranged` crate:

    use deranged;

    type U7 = deranged::RangedU8<0, 127>;

    let mut b = U7::new_static::<64>();

This makes a new data type called `U7` with the minimum value of 0 and the maximum
value of 127, and then declares a new variable with the initial
value 64.

Trying to assign a new value that is outside the range will fail:

    b = U7::new_static::<128>();

You will get a lengthy compile error, starting with:

    error[E0080]: evaluation of `<(deranged::RangedU8<0, 128>, deranged::RangedU8<128, 127>) as deranged::traits::StaticIsValid>::ASSERT` failed

and continuing for some dozen lines, because it originates from one of the macros that
the `deranged` crate uses to define the various subrange types.

I'm not sure I particularly like the ergonomics of the declarations, but failing at compile
time is obviously the right way to handle a situation like this. I did not try to construct
a test of the runtime behavior, because I couldn't figure out how (or if) you could assign
the value of another variable instead of a constant to a type like this.

So maybe, just maybe, this is yet another example of basically going against the grain.

### Just embrace the lack of subrange types

While I admire the efforts of the crate authors, this is a long
way from Ada and declarations like

    type U7 is 0 .. 127;

The path of least resistance seems to be to just use a newtype
and forget about making an actual range-checking data type.
Whenever you use the newtype in a struct, you write functions to
enforce that any incoming values are in the desired range. Then
you just hope that there will not be too many of them.

While I like Rust a lot, this kind of thing really leaves me pining for Ada.
It seems like subrange types are something of a lost art, and attempts to
suggest that they be added are met with indifference (see, for example,
[this discussion on Rust Internals](https://internals.rust-lang.org/t/more-on-ranged-integers/8614)), possibly because
it is not quite clear to all what undisputable advances they do have.

For a language that sometimes seems to take pride in using all the best
bits from computer science (remember Rust originator Graydon Hoare's description
of it as "technology from the past, come to save the future from itself"),
the lack of subrange types seems like a curious omission.

Maybe some day I will come up with a better approximation of a subrange
type in Rust, because they most likely will never be added to the
language. (I wrote this before discovering associated consts; see "One more thing" below.)

### Using the newtype pattern

For leveraging the newtype pattern in Rust, I have found the article
[The Newtype Pattern in Rust](https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html) by Justin Wernick to be
quite helpful. It also brought [the derive_more crate](https://crates.io/crates/derive_more) by Jelte Fennema to my attention.

### One more thing: associated consts

While reading "Programming in Rust" 2nd Edition I came across a feature
of traits (is that a tautology or what) that seems to fit the bill.
You can define associated consts in a trait, but leave their actual values
for the implementor of the trait to define.

For example, I can define a trait with `FIRST`, `LAST`, and `DEFAULT` and related methods
like this:

    pub trait Ranged {
        const FIRST: i32;
        const LAST: i32;
        const DEFAULT: i32;

        fn new(value: i32) -> Self;
        fn value(&self) -> i32;
        fn contains(value: i32) -> bool;
        fn random_value() -> Self;
    }

Then I make a newtype, and make it implement this trait:

    pub struct Algorithm(i32);

    impl Ranged for Algorithm {
        const FIRST: i32 = 1;
        const LAST: i32 = 32;
        const DEFAULT: i32 = 32;

        fn new(value: i32) -> Self {
            if Self::contains(value) {
                Self(value)
            }
            else {
                panic!("expected value in range {}...{}, got {}",
                    Self::FIRST, Self::LAST, value);
            }
        }

        fn value(&self) -> i32 { self.0 }

        fn contains(value: i32) -> bool {
            value >= Self::FIRST && value <= Self::LAST
        }

        fn random_value() -> Self {
            let mut rng = rand::thread_rng();
            Self::new(rng.gen_range(Self::FIRST..=Self::LAST))
        }
    }

How's that for a subrange type? It is immutable, and other newtypes like this
are easy to implement. Essentially the only things that need to change are the name
of the type and the values of the associated consts in the trait implementation.

### The `ranged_impl!` macro

It becomes quite tedious to implement the `Ranged` trait for all the
required types. The code you need to write can be radically shortened
by defining a Rust macro to handle the grunt work.

The `ranged_impl!` macro implements the `Ranged` trait for a given type,
along with the `Default` and `Display` traits. The default value is simply the
`DEFAULT` associated constant, while the displayed value is the value wrapped
by the type.

Remember to install `cargo-expand` if you want to see the result of the
macro expansion:

    cargo install cargo-expand

Then use the `cargo expand` command.

Alternatively, you can also use Rust Analyzer in Visual Studio Code
to recursively expand the macro; see the "Expand macro recursively
at caret" command.

## Leaning on the `From` trait

Most DX7 parameters appear as one byte in the System Exclusive data.
However, they often need a little adjustment to get them from the
7-bit MIDI byte to their respective range. For example, the algorithm
used for a voice is 1...32, but it is stored in the SysEx data as a
value in the range 0...31. Many other parameters are expressed similarly.

The `From<u8>` trait implementation on the `Algorithm` newtype is used to convert
from a System Exclusive data byte representing the algorithm (0...31)
to the actual algorithm value (1...32):

    impl From<u8> for Algorithm {
        fn from(item: u8) -> Self {
            Algorithm(item + 1)  // bring into 1...32
        }
    }

The detune parameter ranges from -7 to +7, so there are 15 discrete values.
In the SysEx data, the value zero denotes -7, while the value 14 is +7:

     0  1  2  3  4  5  6  7  8  9 10 11 12 13 14
    -7 -6 -5 -4 -3 -2 -1  0 +1 +2 +3 +4 +5 +6 +7

To get from the SysEx data byte to the actual value you need to subtract 7.

## Rust considerations

### Operators and EGs

A Yamaha DX7 voice has six operators, while an envelope generator has four rates and
four levels. These numbers are never going to change when dealing with DX7 data.
That is why each operator has its own member in the `Voice` struct, and each rate and
level similarly has its own member in the `Envelope` struct. In my opinion,
this makes the code significantly easier to read and write, compared to traditional
zero-based array/vector indexing. It is more intuitive to write `op1.level` than
`op[0].level`.

### The newtype pattern

The data types of some struct members are defined using the newtype pattern in Rust.

Each voice parameter has an allowed range of values. For example, operator levels
go from 0 to 99 inclusive, detune values are -7 to +7, and so on.

To be able to catch or suppress errors in setting parameter values, I wanted to have
a data type that would restrict its values to a given range, and possibly clamp any
value that falls outside the range. Also, it would convenient to create random values
for a parameter, and maybe also restrict those random values into a subrange.

In Rust, a newtype is "a struct with a single component that you define to get stricter
type checking" ("Programming Rust, 2nd Edition", p. 213). As with any struct, it is
possible to define traits for the newtype. I defined a newtype for every relevant
parameter value, such as `UnsignedLevel` and `Detune`, and defined a simple interface
that allows me to make new values and retrieve them, and also get a byte representation
for System Exclusive messages.

Each newtype is backed by an `i32` value. The parameter values would fit into an `i16`,
but since `i32` is the integer type inferred by default, it is much more convenient
to use. For example, the value of the detune parameter ranges from -7 to 7.
It is represented in System Exclusive messages as a value from 0 to 14.

### Constructing parameter values

Now, when I have newtype like `Detune`, I can implement a method that returns the
range:

    #[derive(Debug, Clone, Copy)]
    pub struct Detune(i32);

    impl Detune {
        pub fn range() -> RangeInclusive<i32> {
            RangeInclusive::new(-7, 7)
        }
    }

When a new `Detune` struct is constructed, the tentative value is checked against
the range:

    impl Detune {
        pub fn new(value: i32) -> Self {
            let range = Detune::range();
            if range.contains(&value) {
                Detune(value)
            }
            else {
                if Self::is_clamped() {
                    Detune(num::clamp(value, *range.start(), *range.end()))
                }
                else {
                    panic!("expected value in range {}...{}, got {}", *range.start(), *range.end(), value);
                }
            }
        }
    }

If the value is out of range, it gets clamped, using the `clamp` function in the
`num` crate. The clamping is controlled by the `is_clamped` function:

    impl Detune {
        pub fn is_clamped() -> bool {
            return true
        }
    }

## Dissection of a DX7 patch

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

