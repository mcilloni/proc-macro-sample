use std::io;
use std::marker::PhantomData;
use std::mem::size_of;

use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use failure::ResultExt;
use num_traits::{FromPrimitive, Signed, ToPrimitive, Unsigned};

use crate::err::ErrorKind::*;
use crate::err::*;

pub trait Dump: Sized {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()>;
}

impl<'a, T: Dump> Dump for &'a T {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        T::dump(*self, write)
    }
}

impl<'a, T: Dump> Dump for &'a mut T {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        T::dump(*self, write)
    }
}

impl<'a, T: Dump> Dump for Box<T> {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        write.dump(self as &T)
    }
}

impl<T: Dump> Dump for Option<T> {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        match self {
            Some(val) => {
                write.dump(&true)?;
                write.dump(val)
            }
            None => write.dump(&false),
        }
    }
}

impl<T: Dump> Dump for Box<[T]> {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        write.dump(&(self as &[T]))
    }
}

impl<T: Dump> Dump for Vec<T> {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        write.dump(&(self as &[T]))
    }
}

impl<'a, T: Dump> Dump for &'a [T] {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        write.dump(&(self.len() as u64))?;

        for elem in self.iter() {
            write.dump(elem)?
        }

        Ok(())
    }
}

impl Dump for bool {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        write.dump(&(*self as u8)) // should serialise true/false as 1/0
    }
}

impl Dump for String {
    fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
        write.write_all(self.as_bytes()).context(FileWrite)?;
        write.write(&[0u8]).context(FileWrite)?;
        Ok(())
    }
}

macro_rules! dump_sint {
    ($ty:ty) => {
        impl Dump for $ty {
            fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
                write.write_leint(*self)
            }
        }
    };
}

macro_rules! dump_uint {
    ($ty:ty) => {
        impl Dump for $ty {
            fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
                write.write_leuint(*self)
            }
        }
    };
}

dump_sint!(i8);
dump_sint!(i16);
dump_sint!(i32);
dump_sint!(i64);
dump_sint!(i128);

dump_uint!(u8);
dump_uint!(u16);
dump_uint!(u32);
dump_uint!(u64);
dump_uint!(u128);

pub trait WriteExt: WriteBytesExt {
    fn dump<D: Dump>(&mut self, to_dump: &D) -> Result<()> {
        to_dump.dump(self)
    }

    fn write_leint<N: Signed + ToPrimitive>(&mut self, n: N) -> Result<()> {
        let prim = n.to_i64().unwrap();

        self.write_int::<LE>(prim, size_of::<N>())
            .context(FileWrite)?;

        Ok(())
    }

    fn write_leuint<N: Unsigned + ToPrimitive>(&mut self, n: N) -> Result<()> {
        let prim = n.to_u64().unwrap();

        self.write_uint::<LE>(prim, size_of::<N>())
            .context(FileWrite)?;

        Ok(())
    }
}

impl<W: io::Write + ?Sized> WriteExt for W {}

macro_rules! impl_dump_array {
    () => (
        impl_dump_array!(0);
    );

    ($n:literal) => (
        impl<T: Dump> Dump for [T;$n] {
            #[allow(non_snake_case)]
            fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
                for el in self {
                    write.dump(el)?;
                }

                Ok(())
            }
        }
    );
}

impl_dump_array!();
impl_dump_array!(1);
impl_dump_array!(2);
impl_dump_array!(3);
impl_dump_array!(4);
impl_dump_array!(5);
impl_dump_array!(6);
impl_dump_array!(7);
impl_dump_array!(8);
impl_dump_array!(9);
impl_dump_array!(10);
impl_dump_array!(11);
impl_dump_array!(12);

macro_rules! impl_dump_tuple {
    () => (
        impl Dump for () {
            fn dump(&self, _: &mut (impl io::Write + ?Sized)) -> Result<()> {
                Ok(())
            }
        }
    );

    ( $($name:ident)+) => (
        impl<$($name: Dump),*> Dump for ($($name,)*) {
            #[allow(non_snake_case)]
            fn dump(&self, write: &mut (impl io::Write + ?Sized)) -> Result<()> {
                let ($(ref $name,)*) = *self;
                $($name.dump(write)?;)*

                Ok(())
            }
        }
    );
}

impl_dump_tuple! {}
impl_dump_tuple! { A }
impl_dump_tuple! { A B }
impl_dump_tuple! { A B C }
impl_dump_tuple! { A B C D }
impl_dump_tuple! { A B C D E }
impl_dump_tuple! { A B C D E F }
impl_dump_tuple! { A B C D E F G }
impl_dump_tuple! { A B C D E F G H }
impl_dump_tuple! { A B C D E F G H I }
impl_dump_tuple! { A B C D E F G H I J }
impl_dump_tuple! { A B C D E F G H I J K }
impl_dump_tuple! { A B C D E F G H I J K L }

pub trait Load: Sized {
    fn load(read: &mut impl io::Read) -> Result<Self>;
}

impl Load for bool {
    fn load(read: &mut impl io::Read) -> Result<Self> {
        read.load::<u8>().map(|n| match n {
            0 => false,
            _ => true,
        })
    }
}

impl<T: Load> Load for Box<T> {
    fn load(read: &mut impl io::Read) -> Result<Self> {
        Ok(Box::new(read.load()?))
    }
}

impl<T: Load> Load for Box<[T]> {
    fn load(read: &mut impl io::Read) -> Result<Self> {
        Vec::load(read).map(Vec::into_boxed_slice)
    }
}

impl<T: Load> Load for Option<T> {
    fn load(read: &mut impl io::Read) -> Result<Self> {
        if read.load()? {
            read.load()
        } else {
            Ok(None)
        }
    }
}

impl Load for String {
    // load() loads a null-terminated string.
    fn load(read: &mut impl io::Read) -> Result<Self> {
        let mut ret = Vec::new();

        let mut buf = [0u8; 1];

        loop {
            read.read_exact(&mut buf).context(FileRead)?;

            if buf[0] == 0u8 {
                break;
            }

            ret.push(buf[0]);
        }

        Ok(Self::from_utf8(ret).context(InvalidUtf8)?)
    }
}

impl<T: Load> Load for Vec<T> {
    fn load(read: &mut impl io::Read) -> Result<Self> {
        let mut ret: Vec<T> = Vec::new();

        for next in read.iter_array()? {
            ret.push(next?);
        }

        Ok(ret)
    }
}

macro_rules! load_sint {
    ($ty:ty) => {
        impl Load for $ty {
            fn load(read: &mut impl io::Read) -> Result<Self> {
                read.read_leint()
            }
        }
    };
}

macro_rules! load_uint {
    ($ty:ty) => {
        impl Load for $ty {
            fn load(read: &mut impl io::Read) -> Result<Self> {
                read.read_leuint()
            }
        }
    };
}

load_sint!(i8);
load_sint!(i16);
load_sint!(i32);
load_sint!(i64);
load_sint!(i128);

load_uint!(u8);
load_uint!(u16);
load_uint!(u32);
load_uint!(u64);
load_uint!(u128);

pub struct ArrayIter<'a, T: Load, R: 'a> {
    read: &'a mut R,

    n_elems: u64,
    read_elems: u64,

    // if the iterator has failed (i.e., next() returned Result::Err),
    // it must stop and return 'None', without being able to change it back.
    failed: bool,

    // phantom marker to keep the T type, without having any T inside
    marker_data: PhantomData<*const T>,
}

impl<'a, T, R> Iterator for ArrayIter<'a, T, R>
where
    T: Load,
    R: 'a + io::Read,
{
    type Item = Result<T>;

    // if the option returned is 'None', the array has been fully read.
    fn next(&mut self) -> Option<Result<T>> {
        if !self.failed && self.read_elems < self.n_elems {
            Some({
                let res = self.read.load();

                // if ok, increment the iterator
                // if not ok, mark it as failed
                match res {
                    Ok(..) => self.read_elems += 1,
                    Err(..) => self.failed = true,
                };

                res
            })
        } else {
            None
        }
    }
}

pub trait ReadExt: ReadBytesExt + Sized {
    fn load<N: Load>(&mut self) -> Result<N> {
        N::load(self)
    }

    fn iter_array<N>(&mut self) -> Result<ArrayIter<N, Self>>
    where
        N: Load,
    {
        // read number of elements
        let n_elems: u64 = self.load()?;

        Ok(ArrayIter {
            read: self,
            n_elems,
            read_elems: 0,
            failed: false,
            marker_data: Default::default(),
        })
    }

    fn read_leint<N: Signed + FromPrimitive>(&mut self) -> Result<N> {
        let res = self.read_int::<LE>(size_of::<N>()).context(FileRead)?;

        Ok(<N>::from_i64(res).unwrap())
    }

    fn read_leuint<N: Unsigned + FromPrimitive>(&mut self) -> Result<N> {
        let res = self.read_uint::<LE>(size_of::<N>()).context(FileRead)?;

        Ok(<N>::from_u64(res).unwrap())
    }
}

impl<R: io::Read> ReadExt for R {}

macro_rules! impl_load_array {
    () => (
        impl_dump_array!(0);
    );

    ($n:literal) => (
        impl<T: Load + Default> Load for [T;$n] {
            #[allow(non_snake_case)]
            fn load(write: &mut (impl io::Write + ?Sized)) -> Result<()> {
                use std::ptr;
                use std::mem;

                let mut arr : [T; $n];


                Ok(arr)
            }
        }
    );
}

impl_load_array!();
impl_load_array!(1);
impl_load_array!(2);
impl_load_array!(3);
impl_load_array!(4);
impl_load_array!(5);
impl_load_array!(6);
impl_load_array!(7);
impl_load_array!(8);
impl_load_array!(9);
impl_load_array!(10);
impl_load_array!(11);
impl_load_array!(12);

macro_rules! impl_load_tuple {
    () => (
        impl Load for () {
            fn load(_: &mut impl io::Read) -> Result<Self> {
                Ok(())
            }
        }
    );

    ($($name:ident)+) => (
        impl<$($name: Load),*> Load for ($($name,)*) {
            #[allow(non_snake_case)]
            fn load(read: &mut impl io::Read) -> Result<Self> {
                $(let $name = read.load()?;)*
                Ok(($($name,)*))
            }
        }
    );
}

impl_load_tuple! {}
impl_load_tuple! { A }
impl_load_tuple! { A B }
impl_load_tuple! { A B C }
impl_load_tuple! { A B C D }
impl_load_tuple! { A B C D E }
impl_load_tuple! { A B C D E F }
impl_load_tuple! { A B C D E F G }
impl_load_tuple! { A B C D E F G H }
impl_load_tuple! { A B C D E F G H I }
impl_load_tuple! { A B C D E F G H I J }
impl_load_tuple! { A B C D E F G H I J K }
impl_load_tuple! { A B C D E F G H I J K L }
