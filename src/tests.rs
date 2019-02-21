use rand::{thread_rng, Rng};

use crate::ext_io::{ReadExt, WriteExt};
use load_dump_derive::*;

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

impl Sample {
    fn get() -> Self {
        use SampleEnum::*;

        let mut rng = thread_rng();

        Self {
            some_box_arr: (0..20).map(|_| rng.gen()).collect(),
            some_enum_arr: [
                One,
                Struct {
                    f1: rng.gen(),
                    f2: "some string".to_owned(),
                },
                Tuple(rng.gen(), rng.gen(), Box::new(One)),
            ],
            some_vec: vec!["one".into(), "two".into(), "three".into()],
        }
    }
}

#[test]
fn test_dump_load() {
    // Get an instance of Sample from somewhere
    let og_inst = Sample::get();
    let mut buf = Vec::new();

    // Turn Vec<_> into an io::Write
    (&mut buf).dump(&og_inst).unwrap();

    // Turn Vec<_> into an io::Read and read back the contents
    let retrieved_inst = (&buf[..]).load().unwrap();

    // The reconstructed instance and the original one will be the same
    assert!(og_inst == dbg!(retrieved_inst));
}
