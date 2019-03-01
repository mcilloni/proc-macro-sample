// Until extern crate self does not get into stable (1.34) this crate won't be able to run
// its tests on stable (given that it cannot reference itself as a crate).
//extern crate self as proc_dump_sample;

extern crate arrayvec;
extern crate byteorder;
extern crate enum_primitive;
extern crate failure;
extern crate num_traits;

extern crate failure_derive;
extern crate load_dump_derive;

mod ext_io;

mod err;

pub use err::*;
pub use ext_io::{Dump, Load, ReadExt, WriteExt};

#[cfg(test)]
extern crate rand;

#[cfg(test)]
mod tests;
