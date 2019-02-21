use rand::{Rng, thread_rng};

use load_dump_derive::*;

#[derive(Dump, Eq, Load, PartialEq)]
enum SampleEnum {
    One,
    Struct { f1: i32, f2: String },
    Tuple(u128, bool, Box<SampleEnum>),
}

#[derive(Dump, Eq, Load, PartialEq)]
struct Sample {
    some_box_arr: Box<[i32]>,
    some_enum: SampleEnum,
    some_vec: Vec<String>,
}

impl Sample {
    fn new() -> Self {
        let rng = thread_rng();

        Self {
            some_box_arr: (0..20).map(|_| rng.gen()).collect(),
            some_enum: 
        }
    }
}

#[test]
fn test_dump_load() {

}

