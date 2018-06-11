use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use std::fmt;
use std::fmt::Debug;

use super::*;

#[derive(Debug)]
pub enum SerError {
    InvalidData(String),
    InvalidType(String),
    Unknown(String),
}

pub trait Serialiser<T> {
    fn id(&self) -> u64;
    fn size_hint(&self) -> Option<usize> {
        None
    }
    fn serialise(&self, v: &T, buf: &mut BufMut) -> Result<(), SerError>;
}

pub trait Serialisable: Debug {
    fn id(&self) -> u64;
    fn size_hint(&self) -> Option<usize>;
    fn serialise(&self, buf: &mut BufMut) -> Result<(), SerError>;
}

impl<T, S> From<(T, S)> for Box<Serialisable>
where
    T: Debug + 'static,
    S: Serialiser<T> + 'static,
{
    fn from(t: (T, S)) -> Self {
        let sv = SerialisableValue { v: t.0, ser: t.1 };
        Box::new(sv) as Box<Serialisable>
    }
}

impl<T> From<T> for Box<Serialisable>
where
    T: Debug + Serialisable + Sized + 'static,
{
    fn from(t: T) -> Self {
        Box::new(t) as Box<Serialisable>
    }
}

pub(crate) struct SerialisableValue<T, S>
where
    T: Debug,
    S: Serialiser<T>,
{
    v: T,
    ser: S,
}

impl<T, S> Serialisable for SerialisableValue<T, S>
where
    T: Debug,
    S: Serialiser<T>,
{
    fn id(&self) -> u64 {
        self.ser.id()
    }
    fn size_hint(&self) -> Option<usize> {
        self.ser.size_hint()
    }
    fn serialise(&self, buf: &mut BufMut) -> Result<(), SerError> {
        self.ser.serialise(&self.v, buf)
    }
}

impl<T, S> Debug for SerialisableValue<T, S>
where
    T: Debug,
    S: Serialiser<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "Serialisable(id={:?},size={:?}, value={:?})",
            self.ser.id(),
            self.ser.size_hint(),
            self.v
        )
    }
}

pub trait Deserialiser<T> {
    fn deserialise(buf: &mut Buf) -> Result<T, SerError>;
}

pub trait Deserialisable<T> {
    fn id(&self) -> u64;
    fn get_deserialised(self) -> Result<T, SerError>;
}

/// Trivial identity Deserialisable
impl<T> Deserialisable<T> for T {
    fn id(&self) -> u64 {
        unimplemented!();
    }
    fn get_deserialised(self) -> Result<T, SerError> {
        Ok(self)
    }
}

/// Trivial heap to Stack Deserialisable
impl<T> Deserialisable<T> for Box<T> {
    fn id(&self) -> u64 {
        unimplemented!();
    }
    fn get_deserialised(self) -> Result<T, SerError> {
        Ok(*self)
    }
}

//struct BufDeserialisable<B: Buf, D: Deserialiser> {
//    buf: B,
//    deser:
//}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(PartialEq, Debug, Clone)]
    struct Test1 {
        i: u64,
    }

    struct T1Ser;
    impl Serialiser<Test1> for T1Ser {
        fn id(&self) -> u64 {
            1
        }
        fn size_hint(&self) -> Option<usize> {
            Some(88)
        }
        fn serialise(&self, v: &Test1, buf: &mut BufMut) -> Result<(), SerError> {
            //buf.put_u64::<BigEndian>(v.i);
            buf.put_u64(v.i);
            for i in 0..10 {
                buf.put_u64(i);
            }
            Result::Ok(())
        }
    }
    impl Deserialiser<Test1> for T1Ser {
        fn deserialise(buf: &mut Buf) -> Result<Test1, SerError> {
            let i = buf.get_u64();
            Result::Ok(Test1 { i })
        }
    }

    #[test]
    fn trivial_deser_equivalence() {
        let t1 = Test1 { i: 42 };
        let t1c = t1.clone();
        //let t1_des: Deserialisable<Test1> = t1;

        //assert_eq!(1, Deserialisable::<Test1>::id(&t1));

        match Deserialisable::<Test1>::get_deserialised(t1) {
            Ok(t2) => assert_eq!(t1c, t2),
            Err(e) => panic!(e),
        }

        let t1b = Box::new(t1c.clone());

        match Deserialisable::<Test1>::get_deserialised(t1b) {
            Ok(t2) => assert_eq!(t1c, t2),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn ser_deser_equivalence() {
        let t1 = Test1 { i: 42 };
        let mut mbuf = BytesMut::with_capacity(64);
        //let mut mbuf = vec![];
        let t1c = t1.clone();
        mbuf.reserve(T1Ser.size_hint().unwrap());
        T1Ser
            .serialise(&t1, &mut mbuf)
            .expect("should have serialised!");
        //println!("Serialised bytes: {:?}", mbuf);
        let mut buf = mbuf.into_buf();
        let t1_res = T1Ser::deserialise(&mut buf);
        match t1_res {
            Ok(t2) => assert_eq!(t1c, t2),
            Err(e) => panic!(e),
        }
        //        match Deserialisable::<Test1>::get(t1) {
        //            Ok(t2) => assert_eq!(t1c, t2),
        //            Err(e) => panic!(e),
        //        }  Err(e) => panic!(e),
        //        }
    }
}
