# proc-macro-sample
This repository contains a short example of how custom `#[derive(MyTrait)]` directives can be implemented using procedural macros in Rust, through the usage of helper libraries such as [quote](https://github.com/dtolnay/syn), [syn](https://github.com/dtolnay/quote) and [proc-macro2](https://github.com/alexcrichton/proc-macro2). This crate contains a simple serialization framework which permits the serialization of arbitrary Rust data types through the usage of two traits, `Dump` and `Load`; these can be automatically implemented using `#derive(Trait)` statements thanks to the procedural macros implemented in `load-dump-derive`.

I decided to extract this code from one of my personal projects and put it here as a reference due to the overall lack of good examples I found around the internet. 

__WARNING__: while this codes mostly works and does what it is supposed to do, you should generally serialize Rust datatypes using a standard format through Serde, which is (almost) always the wisest choice; do not use custom serialization formats unless your use case really requires them.

## `load-dump-derive`

`load-dump-derive` implements a custom derive logic for the traits `Dump` and `Load`, using the types and facilities provided by `quote`, `syn` and `proc-macro2`. This crate is capable of automatically derive an implementation of both for any complex type (containing any arbitrarily nested `struct`, `enum` or `tuple`) as long as each one of its members is `Load`-able/`Dump`-able itself.

## The `proc-macro-sample` crate

`proc-macro-sample` defines `Dump` and `Load`, plus implementations of these two traits for basic Rust types (such as integers, `String`, ...) and containers of already dumpable/loadable ones, including the likes of `Box<T>`, `Box<[T]>`, `Vec<T>`, arrays, tuples, etc. 

## Example

The snippet below represents the desired usage of the derive procedural macros we wrote:

```rust
#[derive(Debug, Dump, Eq, Load, PartialEq)]
enum SampleEnum {
    One,
    Struct { f1: i32, f2: String },
    Tuple(u128, bool, Box<SampleEnum>),
}

#[derive(Debug, Dump, Eq, Load, PartialEq)]
struct Sample {
    some_box_arr: Box<[i32]>,
    some_enum_arr: [SampleEnum; 3],
    some_vec: Vec<String>,
}

fn main() {
    // Get an instance of Sample from somewhere
    let og_inst = Sample::get(); 
    let mut buf = Vec::new();

    // Turn Vec<_> into an io::Write
    (&mut buf).dump(&og_inst).unwrap();

    // Turn Vec<_> into an io::Read and read back the contents
    let retrieved_inst = (&buf[..]).load().unwrap();

    // The reconstructed instance and the original one will be the same
    assert!(og_inst == retrieved_inst);
}
```

## License

BSD (2-clause FreeBSD license)