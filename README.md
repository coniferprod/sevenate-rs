# sevenate-rs

Rust library for working with Yamaha DX7 patches.

## Usage

The crate contains several types that implement the `Ranged` trait
from the SyxPack crate. Each of them is newtype, followed by the trait implementation.
For example, the `Algorithm` type represents values from 1 to 32 inclusive.

When making a value of a doman type from a MIDI System Exclusive data byte,
use the `parse_or_default` function in the SyxPack crate:

    let alg = parse_or_default::<Algorithm>(data[134])

This will perform the necessary conversion using the `Encoding` trait from
SyxPack.

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
However, since `i32` is the integer type inferred by default, it is much more convenient
to use.

Each domain type is a newtype ("a struct with a single component that you define to get stricter
type checking" ("Programming Rust, 2nd Edition", p. 213)).

The `Encoding` trait can be used to perform any adjustments when reading from
or writing to System Exclusive data files. The default implementations of the
trait methods perform an identity transformation, so you only need to implement
this trait if the domain type value needs adjustments.

## History and rationale

For the history and rationale of the `sevenate-rs` crate
see Flecks of Rust #12, [Subrange types in Rust](https://www.coniferproductions.com/rust/flecks/12/).
It contains most of the material that used to be in this `README`.
