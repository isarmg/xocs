use crate::error::{Error, Result};
use crate::ser::flavors::{Cobs, Flavor, Slice};
use serde::Serialize;

#[cfg(feature = "heapless")]
use crate::ser::flavors::HVec;

#[cfg(feature = "heapless")]
use heapless::Vec;

#[cfg(feature = "alloc")]
use crate::ser::flavors::AllocVec;

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::ser::serializer::Serializer;

pub mod flavors;
pub(crate) mod serializer;

/// Serialize a `T` to the given slice, with the resulting slice containing
/// data in a serialized then COBS encoded format. The terminating sentinel
/// `0x00` byte is included in the output buffer.
///
/// When successful, this function returns the slice containing the
/// serialized and encoded message.
///
/// ## Example
///
/// ```rust
/// use postcard::to_slice_cobs;
/// let mut buf = [0u8; 32];
///
/// let used = to_slice_cobs(&false, &mut buf).unwrap();
/// assert_eq!(used, &[0x01, 0x01, 0x00]);
///
/// let used = to_slice_cobs("1", &mut buf).unwrap();
/// assert_eq!(used, &[0x03, 0x01, b'1', 0x00]);
///
/// let used = to_slice_cobs("Hi!", &mut buf).unwrap();
/// assert_eq!(used, &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
///
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let used = to_slice_cobs(data, &mut buf).unwrap();
/// assert_eq!(used, &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
/// ```
pub fn to_slice_cobs<'a, 'b, T>(value: &'b T, buf: &'a mut [u8]) -> Result<&'a mut [u8]>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Cobs<Slice<'a>>, &'a mut [u8]>(
        value,
        Cobs::try_new(Slice::new(buf))?,
    )
}

/// Serialize a `T` to the given slice, with the resulting slice containing
/// data in a serialized format.
///
/// When successful, this function returns the slice containing the
/// serialized message
///
/// ## Example
///
/// ```rust
/// use postcard::to_slice;
/// let mut buf = [0u8; 32];
///
/// let used = to_slice(&true, &mut buf).unwrap();
/// assert_eq!(used, &[0x01]);
///
/// let used = to_slice("Hi!", &mut buf).unwrap();
/// assert_eq!(used, &[0x03, b'H', b'i', b'!']);
///
/// // NOTE: postcard handles `&[u8]` and `&[u8; N]` differently.
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let used = to_slice(data, &mut buf).unwrap();
/// assert_eq!(used, &[0x04, 0x01, 0x00, 0x20, 0x30]);
///
/// let data: &[u8; 4] = &[0x01u8, 0x00, 0x20, 0x30];
/// let used = to_slice(data, &mut buf).unwrap();
/// assert_eq!(used, &[0x01, 0x00, 0x20, 0x30]);
/// ```
pub fn to_slice<'a, 'b, T>(value: &'b T, buf: &'a mut [u8]) -> Result<&'a mut [u8]>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Slice<'a>, &'a mut [u8]>(value, Slice::new(buf))
}

/// Serialize a `T` to a `heapless::Vec<u8>`, with the `Vec` containing
/// data in a serialized then COBS encoded format. The terminating sentinel
/// `0x00` byte is included in the output `Vec`.
///
/// ## Example
///
/// ```rust
/// use postcard::to_vec_cobs;
/// use heapless::Vec;
/// use core::ops::Deref;
///
/// let ser: Vec<u8, 32> = to_vec_cobs(&false).unwrap();
/// assert_eq!(ser.deref(), &[0x01, 0x01, 0x00]);
///
/// let ser: Vec<u8, 32> = to_vec_cobs("Hi!").unwrap();
/// assert_eq!(ser.deref(), &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
///
/// // NOTE: postcard handles `&[u8]` and `&[u8; N]` differently.
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, 32> = to_vec_cobs(data).unwrap();
/// assert_eq!(ser.deref(), &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
///
/// let data: &[u8; 4] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, 32> = to_vec_cobs(data).unwrap();
/// assert_eq!(ser.deref(), &[0x02, 0x01, 0x03, 0x20, 0x30, 0x00]);
/// ```
#[cfg(feature = "heapless")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless")))]
pub fn to_vec_cobs<T, const B: usize>(value: &T) -> Result<Vec<u8, B>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Cobs<HVec<B>>, Vec<u8, B>>(value, Cobs::try_new(HVec::default())?)
}

/// Serialize a `T` to a `heapless::Vec<u8>`, with the `Vec` containing
/// data in a serialized format.
///
/// ## Example
///
/// ```rust
/// use postcard::to_vec;
/// use heapless::Vec;
/// use core::ops::Deref;
///
/// let ser: Vec<u8, 32> = to_vec(&true).unwrap();
/// assert_eq!(ser.deref(), &[0x01]);
///
/// let ser: Vec<u8, 32> = to_vec("Hi!").unwrap();
/// assert_eq!(ser.deref(), &[0x03, b'H', b'i', b'!']);
///
/// // NOTE: postcard handles `&[u8]` and `&[u8; N]` differently.
/// let data: &[u8] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, 32> = to_vec(data).unwrap();
/// assert_eq!(ser.deref(), &[0x04, 0x01, 0x00, 0x20, 0x30]);
///
/// let data: &[u8; 4] = &[0x01u8, 0x00, 0x20, 0x30];
/// let ser: Vec<u8, 32> = to_vec(data).unwrap();
/// assert_eq!(ser.deref(), &[0x01, 0x00, 0x20, 0x30]);
/// ```
#[cfg(feature = "heapless")]
#[cfg_attr(docsrs, doc(cfg(feature = "heapless")))]
pub fn to_vec<T, const B: usize>(value: &T) -> Result<Vec<u8, B>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, HVec<B>, Vec<u8, B>>(value, HVec::default())
}

/// Serialize a `T` to a `std::vec::Vec<u8>`.
///
/// ## Example
///
/// ```rust
/// use postcard::to_stdvec;
///
/// let ser: Vec<u8> = to_stdvec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_stdvec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "use-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "use-std")))]
#[inline]
pub fn to_stdvec<T>(value: &T) -> Result<std::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_allocvec(value)
}

/// Serialize and COBS encode a `T` to a `std::vec::Vec<u8>`.
///
/// The terminating sentinel `0x00` byte is included in the output.
///
/// ## Example
///
/// ```rust
/// use postcard::to_stdvec_cobs;
///
/// let ser: Vec<u8> = to_stdvec_cobs(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x02, 0x01, 0x00]);
///
/// let ser: Vec<u8> = to_stdvec_cobs("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
/// ```
#[cfg(feature = "use-std")]
#[cfg_attr(docsrs, doc(cfg(feature = "use-std")))]
#[inline]
pub fn to_stdvec_cobs<T>(value: &T) -> Result<std::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_allocvec_cobs(value)
}

/// Serialize a `T` to an `alloc::vec::Vec<u8>`.
///
/// ## Example
///
/// ```rust
/// use postcard::to_allocvec;
///
/// let ser: Vec<u8> = to_allocvec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_allocvec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn to_allocvec<T>(value: &T) -> Result<alloc::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, AllocVec, alloc::vec::Vec<u8>>(value, AllocVec::new())
}

/// Serialize and COBS encode a `T` to an `alloc::vec::Vec<u8>`.
///
/// The terminating sentinel `0x00` byte is included in the output.
///
/// ## Example
///
/// ```rust
/// use postcard::to_allocvec_cobs;
///
/// let ser: Vec<u8> = to_allocvec_cobs(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x02, 0x01, 0x00]);
///
/// let ser: Vec<u8> = to_allocvec_cobs("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x05, 0x03, b'H', b'i', b'!', 0x00]);
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn to_allocvec_cobs<T>(value: &T) -> Result<alloc::vec::Vec<u8>>
where
    T: Serialize + ?Sized,
{
    serialize_with_flavor::<T, Cobs<AllocVec>, alloc::vec::Vec<u8>>(
        value,
        Cobs::try_new(AllocVec::new())?,
    )
}

/// Serialize a `T` to a [`core::iter::Extend`],
/// ## Example
///
/// ```rust
/// use postcard::to_extend;
/// let mut vec = Vec::new();
///
/// let ser = to_extend(&true, vec).unwrap();
/// let vec = to_extend("Hi!", ser).unwrap();
/// assert_eq!(&vec[0..5], &[0x01, 0x03, b'H', b'i', b'!']);
/// ```
pub fn to_extend<T, W>(value: &T, writer: W) -> Result<W>
where
    T: Serialize + ?Sized,
    W: core::iter::Extend<u8>,
{
    serialize_with_flavor::<T, _, _>(value, flavors::ExtendFlavor::new(writer))
}

/// Serialize a `T` to a [`std::io::Write`],
/// ## Example
///
/// ```rust
/// use postcard::to_io;
/// let mut buf: [u8; 32] = [0; 32];
/// let mut writer: &mut [u8] = &mut buf;
///
/// let ser = to_io(&true, &mut writer).unwrap();
/// to_io("Hi!", ser).unwrap();
/// assert_eq!(&buf[0..5], &[0x01, 0x03, b'H', b'i', b'!']);
/// ```
#[cfg(feature = "use-std")]
pub fn to_io<T, W>(value: &T, writer: W) -> Result<W>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    serialize_with_flavor::<T, _, _>(value, flavors::io::WriteFlavor::new(writer))
}

/// `serialize_with_flavor()` has three generic parameters, `T, F, O`.
///
/// * `T`: This is the type that is being serialized
/// * `S`: This is the Storage that is used during serialization
/// * `O`: This is the resulting storage type that is returned containing the serialized data
///
/// For more information about how Flavors work, please see the
/// [`flavors` module documentation](./flavors/index.html).
///
/// ```rust
/// use postcard::{
///     serialize_with_flavor,
///     ser_flavors::{Cobs, Slice},
/// };
///
/// let mut buf = [0u8; 32];
///
/// let data: &[u8] = &[0x01, 0x00, 0x20, 0x30];
/// let buffer = &mut [0u8; 32];
/// let res = serialize_with_flavor::<[u8], Cobs<Slice>, &mut [u8]>(
///     data,
///     Cobs::try_new(Slice::new(buffer)).unwrap(),
/// ).unwrap();
///
/// assert_eq!(res, &[0x03, 0x04, 0x01, 0x03, 0x20, 0x30, 0x00]);
/// ```
pub fn serialize_with_flavor<T, S, O>(value: &T, storage: S) -> Result<O>
where
    T: Serialize + ?Sized,
    S: Flavor<Output = O>,
{
    let mut serializer = Serializer { output: storage };
    value.serialize(&mut serializer)?;
    serializer
        .output
        .finalize()
        .map_err(|_| Error::SerializeBufferFull)
}

#[path = "../../../../../../test/foundation/format/binary-record/unit/src/ser/mod/test.rs"]
#[cfg(feature = "heapless")]
#[cfg(test)]
mod test;
