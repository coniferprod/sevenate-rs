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
I tried to implement a trait that has `const` parameters for the start
and end of the range. That ended up looking really ugly and was
cumbersome to use:

    /// A simple struct for wrapping an `i32`
    /// with const generic parameters to limit
    /// the range of allowed values.
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct RangedInteger<const MIN: i32, const MAX: i32> {
        value: i32,
    }

I wanted to make it easy to create values from types that
implement this trait, so I added a constructor that uses the
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

When I want to make a type that implements this trait, this is what
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

### Embrace the lack of subrange types

While I admire the efforts of the crate authors, this is a long
way from Ada and declarations like

    type U7 is 0 .. 127;

The path of least resistance seems to be to just use a newtype
and forget about making an actual range-checking data type.
Whenever you use the newtype in a struct, you write functions to
enforce that any incoming values are in the desired range. Then
you just hope that there will not be too many of them.

While I like Rust a lot, this kind of thing really leaves me pining for Ada.
It seems like subrange types are a lost art, and attempts to
suggest that they be added are met with indifference, possibly because
it is not quite clear to all what undisputable advances they do have.

Maybe some day I will come up with a better approximation of a subrange
type in Rust, because they most likely will never be added to the
language.

### Using the newtype pattern

For leveraging the newtype pattern in Rust, I have found the article 
[The Newtype Pattern in Rust](https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html) by Justin Wernick to be 
quite helpful. It also brought [the derive_more crate](https://crates.io/crates/derive_more) by Jelte Fennema to my attention.


## Leaning on the From trait

Most DX7 parameters appear as one byte in the System Exclusive data.
However, they often need a little adjustment to get them from the
7-bit MIDI byte to their respective range. For example, the algorithm
used for a voice is 1...32, but it is stored in the SysEx data as a
value in the range 0...31. Many other parameters are expressed similarly.

I'm using a `From<u8>` trait implementation on the `Algorithm` newtype to convert
from a System Excusive data byte to the actual algorithm value:

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
