extern crate arrayvec;
extern crate byteorder;
extern crate enum_primitive;
extern crate failure;
extern crate num_traits;

extern crate failure_derive;
extern crate load_dump_derive;

mod ext_io;

pub mod err;

pub use ext_io::{Dump, Load, ReadExt, WriteExt};

#[cfg(test)]
extern crate rand;

#[cfg(test)]
mod tests;
