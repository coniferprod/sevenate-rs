# sevenate-rs

Rust library for working with Yamaha DX7 patches.

## Usage

The crate contains several types that implement the `Ranged` trait.
Each of them is newtype, followed by the trait implementation.
For example, the `Algorithm` type represents values from 1 to 32 inclusive:

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

Use the constructor of the domain type when making a value:

    let alg = Algorithm::new(data[134] as i32 + 1);  // 0...31 to 1...32

If you specify an initial value that is outside the range `FIRST`..=`LAST`
the constructor will panic, so you should first check that the value fits
using the `contains` method:

    let alg_value = 42;
    let algorithm: Algorithm;
    if Algorithm::contains(alg_value) {
        algorithm = Algorithm::new(alg_value);
        println!("algorithm = {}", algorithm);
    } else {
        eprintln!("Bad value for algorithm: {}", alg_value);
    }

This might be better expressed with an optional, though.

### The `ranged_impl!` macro

It becomes quite tedious to implement the `Ranged` trait for all the
required types, so the `ranged_impl!` macro is defined to handle the
grunt work. It generates an implementation of the `Ranged` trait for a given type,
along with the `Default` and `Display` traits. The default value is the
`DEFAULT` associated constant, while the displayed value is the actual
value wrapped by the type.

To create a new domain type, make a newtype and use the `ranged_impl!` macro:

    /// Algorithm (1...32)
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct Algorithm(i32);

    crate::ranged_impl!(Algorithm, 1, 32, 32);  // first, last, default

Remember to install `cargo-expand` if you want to see the result of the
macro expansion:

    cargo install cargo-expand

Then use the `cargo expand` command to view the generated code.

Alternatively, you can also use Rust Analyzer in Visual Studio Code
to recursively expand the macro; see the _Expand macro recursively
at caret_ command.

## Implementation

The `Ranged` trait defines the basis for constrained integer types,
with associated consts. This allows you leave the actual values of
the consts to be determined by the implementer of the trait. In this
case, the domain types defined in the crate implement `Ranged`.

Implementations of the `Ranged` trait wrap a single `i32` value, since
that was the lowest common denominator between the various values of
the domain types. This could be made generic, but this seems to work
for now and is easy to use. The parameter values would fit into an `i16`;
for example, the value of the detune parameter ranges from -7 to 7, and
it is represented in System Exclusive messages as a value from 0 to 14.
Since `i32` is the integer type inferred by default, it is much more convenient
to use.

Each domain type is a newtype ("a struct with a single component that you define to get stricter
type checking" ("Programming Rust, 2nd Edition", p. 213)).

### Leaning on the `From` trait

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

## History and rationale

For the history and rationale of the `sevenate-rs` crate
see Flecks of Rust #12, [Subrange types in Rust](https://www.coniferproductions.com/rust/flecks/12/).
It contains most of the material that used to be in this `README`.
