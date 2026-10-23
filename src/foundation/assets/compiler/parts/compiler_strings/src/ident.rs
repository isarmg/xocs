//! Identifier string type with precomputed hash.

use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
    hash::{Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
    ptr::NonNull,
    slice, str,
};

use crate::syntax_arena::{
    Allocator, ArenaStringBuilder, CloneIn, CloneInSemanticIds, Dummy, FromIn, GetAllocator,
};
#[cfg(any())]
use crate::syntax_serialization::{ESTree, JsonSafeString, Serializer as ESTreeSerializer};
#[cfg(any())]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::compiler_strings::{
    CompactStr, Str,
    ident_hasher::{IdentBuildHasher, ident_hash},
};

/// A packed representation of `len` and `hash` for `Ident` - 64-bit platforms version.
///
/// Stored as a single `u64`, with `len` in lower 32 bits, `hash` in upper 32 bits.
#[cfg(target_pointer_width = "64")]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct LenAndHash(u64);

#[cfg(target_pointer_width = "64")]
#[expect(clippy::inline_always)] // All methods are trivial
impl LenAndHash {
    #[inline(always)]
    const fn new(len: u32, hash: u32) -> Self {
        Self((len as u64) | ((hash as u64) << 32))
    }

    /// Create a `LenAndHash` with a `0` hash directly from a `usize` `len`.
    ///
    /// Converting the `usize` straight to `u64` (rather than truncating to `u32` and widening
    /// back) is a no-op, because identifier lengths always fit in 32 bits, so the top 32 bits
    /// (the hash) are `0`. This lets [`Ident::new_unhashed`] compile to a no-op reinterpret of
    /// `&str`, which has identical layout.
    #[inline(always)]
    const fn unhashed(len: usize) -> Self {
        Self(len as u64)
    }

    #[expect(clippy::cast_possible_truncation)]
    #[inline(always)]
    const fn len(self) -> u32 {
        self.0 as u32
    }

    #[inline(always)]
    const fn hash(self) -> u32 {
        (self.0 >> 32) as u32
    }

    #[inline(always)]
    const fn to_u64(self) -> u64 {
        self.0
    }
}

/// A packed representation of `len` and `hash` for `Ident` - 32-bit platforms version.
///
/// Stored as 2 separate `u32`s for `len` and `hash`.
/// This is preferable on 32-bit platforms, because it has alignment 4, so `Ident` is 12 bytes, not 16.
#[cfg(not(target_pointer_width = "64"))]
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct LenAndHash {
    len: u32,
    hash: u32,
}

#[cfg(not(target_pointer_width = "64"))]
#[expect(clippy::inline_always)] // All methods are trivial
impl LenAndHash {
    #[inline(always)]
    const fn new(len: u32, hash: u32) -> Self {
        Self { len, hash }
    }

    /// Create a `LenAndHash` with a `0` hash directly from a `usize` `len`.
    ///
    /// See the 64-bit version of this method for why it takes a `usize`.
    #[expect(clippy::cast_possible_truncation)]
    #[inline(always)]
    const fn unhashed(len: usize) -> Self {
        Self { len: len as u32, hash: 0 }
    }

    #[inline(always)]
    const fn len(self) -> u32 {
        self.len
    }

    #[inline(always)]
    const fn hash(self) -> u32 {
        self.hash
    }

    #[inline(always)]
    const fn to_u64(self) -> u64 {
        (self.len as u64) | ((self.hash as u64) << 32)
    }
}

/// An identifier string for oxc_allocator with a precomputed hash.
///
/// Stores a pointer to string data, length, and a precomputed hash for fast
/// HashMap lookups and equality comparisons.
///
/// On 64-bit platforms, length and hash are packed into a single `u64` for
/// maximum performance (16 bytes total, same as `&str`).
/// On 32-bit platforms, they are stored as separate `u32` fields (12 bytes total).
///
/// Use [CompactStr] with [Ident::to_compact_str] or [Ident::into_compact_str] for
/// the lifetimeless form.
#[repr(C)]
pub struct Ident<'a> {
    ptr: NonNull<u8>,
    len_and_hash: LenAndHash,
    _marker: PhantomData<&'a str>,
}

/// Create a new [`Ident`] from a string slice.
///
/// This is a const fn that computes the hash at compile time when possible.
/// Use this for strings that already have the correct lifetime
/// (e.g. arena-allocated strings, or `'static` string literals).
///
/// This function has to be public so it can be used in `static_ident!` macro,
/// but it is not intended to be used directly. Only exported under the `__internal` module.
#[expect(clippy::inline_always, clippy::cast_possible_truncation)]
#[inline(always)]
pub const fn new_const_ident(s: &str) -> Ident<'_> {
    let bytes = s.as_bytes();
    let len = bytes.len() as u32;
    let hash = ident_hash(bytes);
    let ptr = NonNull::from_ref(bytes).cast::<u8>();
    // SAFETY: `ptr` points to a `&str`, with length `len`.
    // The `&str` has lifetime `'a`, so the memory backing the `&str` is immutable for lifetime `'a`.
    // `hash` was computed with `ident_hash`.
    unsafe { Ident::from_raw(ptr, len, hash) }
}

impl<'a> Ident<'a> {
    /// Allocate provided `&str` into arena, and return an [`Ident<'a>`].
    #[inline]
    pub fn from_str_in(s: &str, allocator: &impl GetAllocator<'a>) -> Self {
        new_const_ident(allocator.allocator().alloc_str(s))
    }

    /// Create an [`Ident`] without precomputing its hash (hash field is `0`).
    ///
    /// `Eq` and `Hash` include the stored hash, so an unhashed `Ident` does not compare equal to,
    /// or hash the same as, a hashed `Ident` of the same string. Only use this when nothing
    /// downstream relies on `Ident` hashing (e.g. parse-only pipelines that skip semantic analysis).
    ///
    /// On 64-bit platforms this compiles to a no-op: an unhashed `Ident` has the same layout and
    /// bit representation as `&str` (a pointer plus the `usize` length, whose top 32 bits are the
    /// `0` hash), so no work is done beyond reinterpreting the `&str`.
    #[inline]
    pub const fn new_unhashed(s: &'a str) -> Self {
        let bytes = s.as_bytes();
        let ptr = NonNull::from_ref(bytes).cast::<u8>();
        // `ptr` points to a `&str` with lifetime `'a`, of length `bytes.len()`, whose memory is
        // immutable for lifetime `'a`. The stored hash is `0` (unhashed).
        Self { ptr, len_and_hash: LenAndHash::unhashed(bytes.len()), _marker: PhantomData }
    }

    /// Create an [`Ident`] from raw components.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must point to the start of a valid UTF-8 string, of length `len`.
    /// * The memory pointed to `len` bytes starting at `ptr` must be valid for reads and immutable for lifetime `'a`.
    /// * `hash` must be an accurate hash of the string, calculated with `ident_hash`,
    ///   or `0` for an unhashed `Ident` (see [`Ident::new_unhashed`]).
    #[inline]
    const unsafe fn from_raw(ptr: NonNull<u8>, len: u32, hash: u32) -> Self {
        Self { ptr, len_and_hash: LenAndHash::new(len, hash), _marker: PhantomData }
    }

    /// Get the length of the identifier string.
    #[inline]
    const fn ident_len(&self) -> u32 {
        self.len_and_hash.len()
    }

    /// Get the precomputed hash value.
    #[inline]
    const fn ident_hash_value(&self) -> u32 {
        self.len_and_hash.hash()
    }

    /// Get an [`Ident`] containing the empty string (`""`).
    #[inline]
    pub const fn empty() -> Self {
        // SAFETY: Any pointer is valid for reads of 0 bytes, for the lifetime of the program.
        // `hash` is computed with `ident_hash`.
        unsafe { Self::from_raw(NonNull::dangling(), 0, ident_hash(b"")) }
    }

    /// Borrow a string slice.
    #[expect(clippy::inline_always)]
    #[inline(always)] // Hot path — must be inlined
    pub fn as_str(&self) -> &'a str {
        let len = self.ident_len() as usize;
        // SAFETY: The pointer and length are valid because they came from a valid &str.
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(self.ptr.as_ptr(), len)) }
    }

    /// Convert this [`Ident`] into a [`Str`].
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    pub fn as_arena_str(&self) -> Str<'a> {
        Str::from(self.as_str())
    }

    /// Convert this [`Ident`] into a [`String`].
    ///
    /// This is the explicit form of [`Into<String>`], which [`Ident`] also implements.
    #[inline]
    pub fn into_string(self) -> String {
        String::from(self.as_str())
    }

    /// Convert this [`Ident`] into a [`CompactStr`].
    ///
    /// This is the explicit form of [`Into<CompactStr>`], which [`Ident`] also implements.
    #[inline]
    pub fn into_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    /// Convert this [`Ident`] into a [`CompactStr`] without consuming `self`.
    #[inline]
    pub fn to_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    /// Create new [`Ident`] from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`.
    ///
    /// # Panics
    ///
    /// Panics if the sum of length of all strings exceeds `isize::MAX`.
    //
    // `#[inline(always)]` because want compiler to be able to optimize where some of `strings`
    // are statically known. See `Allocator::alloc_concat_strs_array`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn from_strs_array_in<const N: usize>(
        strings: [&str; N],
        allocator: &impl GetAllocator<'a>,
    ) -> Ident<'a> {
        Self::from(allocator.allocator().alloc_concat_strs_array(strings))
    }

    /// Convert a [`Cow<'a, str>`] to an [`Ident<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns an `Ident` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Ident`.
    #[inline]
    pub fn from_cow_in(value: &Cow<'a, str>, allocator: &impl GetAllocator<'a>) -> Ident<'a> {
        match value {
            Cow::Borrowed(s) => Ident::from(*s),
            Cow::Owned(s) => Ident::from_str_in(s, allocator),
        }
    }
}

// SAFETY: `Ident` is conceptually equivalent to `&str`, which is Send + Sync.
// `NonNull` is !Send/!Sync, but `Ident` only stores a pointer to borrowed data.
unsafe impl Send for Ident<'_> {}
// SAFETY: See above.
unsafe impl Sync for Ident<'_> {}

impl Deref for Ident<'_> {
    type Target = str;

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Ident<'_> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// We can't derive `Clone` or `Copy` because `NonNull` prevents it.
// The explicit impl is needed for `Copy` to work.
#[expect(clippy::expl_impl_clone_on_copy)]
impl Clone for Ident<'_> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for Ident<'_> {}

impl<'a> From<&'a str> for Ident<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        new_const_ident(s)
    }
}

impl<'alloc> From<ArenaStringBuilder<'alloc>> for Ident<'alloc> {
    #[inline]
    fn from(s: ArenaStringBuilder<'alloc>) -> Self {
        Self::from(s.into_str())
    }
}

impl<'a> From<Ident<'a>> for &'a str {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: Ident<'a>) -> Self {
        s.as_str()
    }
}

impl<'a> From<Ident<'a>> for Str<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: Ident<'a>) -> Self {
        s.as_arena_str()
    }
}

impl<'a> From<Str<'a>> for Ident<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: Str<'a>) -> Self {
        Self::from(s.as_str())
    }
}

impl From<Ident<'_>> for CompactStr {
    #[inline]
    fn from(val: Ident<'_>) -> Self {
        val.into_compact_str()
    }
}

impl From<Ident<'_>> for String {
    #[inline]
    fn from(val: Ident<'_>) -> Self {
        val.into_string()
    }
}

impl<'a> From<Ident<'a>> for Cow<'a, str> {
    #[inline]
    fn from(value: Ident<'a>) -> Self {
        Cow::Borrowed(value.as_str())
    }
}

impl PartialEq for Ident<'_> {
    /// Fast-reject equality: compare packed len+hash first, then bytes.
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        self.len_and_hash == other.len_and_hash && self.as_str() == other.as_str()
    }
}

impl Eq for Ident<'_> {}

impl PartialEq<str> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<Ident<'_>> for &str {
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<Ident<'_>> for Cow<'_, str> {
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl PartialEq<&Ident<'_>> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &&Ident<'_>) -> bool {
        self == *other
    }
}

impl PartialEq<Str<'_>> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &Str<'_>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Hash for Ident<'_> {
    /// Write the precomputed packed len+hash as a single u64.
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_u64(self.len_and_hash.to_u64());
    }
}

impl Display for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_str(), f)
    }
}

impl Debug for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.as_str(), f)
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Ident<'_> {
    type Cloned = Ident<'new_alloc>;

    /// Clone the identifier into a new allocator, preserving the precomputed hash.
    #[inline]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        let s = allocator.alloc_str(self.as_str());
        let ptr = NonNull::from_ref(s).cast::<u8>();
        // SAFETY: `ptr` points to a `&str`, with length `self.ident_len()`.
        // The `&str` was just allocated and so inherits the allocator lifetime.
        // `hash` is taken from an existing `Ident` containing the same string,
        // which was originally calculated with `ident_hash`.
        unsafe { Ident::from_raw(ptr, self.ident_len(), self.ident_hash_value()) }
    }
}

impl<'a> Dummy<'a> for Ident<'a> {
    /// Create a dummy [`Ident`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        Ident::empty()
    }
}

impl<'alloc> FromIn<'alloc, &str> for Ident<'alloc> {
    #[inline]
    fn from_in(s: &str, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(s))
    }
}

impl<'alloc> FromIn<'alloc, String> for Ident<'alloc> {
    #[inline]
    fn from_in(s: String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, &String> for Ident<'alloc> {
    #[inline]
    fn from_in(s: &String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, Cow<'_, str>> for Ident<'alloc> {
    #[inline]
    fn from_in(s: Cow<'_, str>, allocator: &'alloc Allocator) -> Self {
        Self::from_in(&*s, allocator)
    }
}

/// Allows looking up an `Ident`-keyed hashbrown map with a `&str` key,
/// without requiring `Ident: Borrow<str>`.
impl crate::syntax_arena::hash_map::Equivalent<Ident<'_>> for str {
    #[inline]
    fn equivalent(&self, key: &Ident<'_>) -> bool {
        self == key.as_str()
    }
}

#[cfg(any())]
impl Serialize for Ident<'_> {
    #[inline] // Because it just delegates
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(self.as_str(), serializer)
    }
}

#[cfg(any())]
impl ESTree for Ident<'_> {
    #[inline] // Because it just delegates
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        // `Ident`s are always JSON-safe
        ESTree::serialize(&JsonSafeString(self.as_str()), serializer);
    }
}

/// Hash map keyed by [`Ident`], using precomputed ident hash.
pub type IdentHashMap<'a, V> = hashbrown::HashMap<Ident<'a>, V, IdentBuildHasher>;

/// Arena-allocated hash map keyed by [`Ident`], using precomputed ident hash.
pub type ArenaIdentHashMap<'alloc, V> =
    crate::syntax_arena::ArenaHashMap<'alloc, Ident<'alloc>, V, IdentBuildHasher>;

/// Hash set of [`Ident`], using precomputed ident hash.
pub type IdentHashSet<'a> = hashbrown::HashSet<Ident<'a>, IdentBuildHasher>;

/// Arena-allocated hash set of [`Ident`], using precomputed ident hash.
pub type ArenaIdentHashSet<'alloc> =
    crate::syntax_arena::ArenaHashSet<'alloc, Ident<'alloc>, IdentBuildHasher>;

/// Creates an [`Ident<'static>`] for a string literal, evaluated at compile time.
///
/// ```
/// use oxc_str::static_ident;
///
/// let ident = static_ident!("require");
/// assert_eq!(ident.as_str(), "require");
/// ```
///
/// Can also be used in const context:
///
/// ```
/// use oxc_str::{Ident, static_ident};
///
/// const REQUIRE: Ident<'static> = static_ident!("require");
/// assert_eq!(REQUIRE.as_str(), "require");
/// ```
///
/// Only accepts string literals, not variables:
///
/// ```compile_fail
/// use oxc_str::static_ident;
///
/// let s = "hello";
/// let ident = static_ident!(s);
/// ```
#[macro_export]
macro_rules! static_ident {
    ($s:literal) => {
        $crate::compiler_strings::__internal::new_const_ident($s)
    };
}

/// Creates an [`Ident`] using interpolation of runtime expressions.
///
/// Identical to [`std`'s `format!` macro](std::format), except:
///
/// * First argument is the allocator.
/// * Produces an [`Ident`] instead of a [`String`].
///
/// The string is built in the arena, without allocating an intermediate `String`.
///
/// # Panics
///
/// Panics if a formatting trait implementation returns an error.
///
/// # Example
///
/// ```
/// use crate::syntax_arena::Allocator;
/// use oxc_str::format_ident;
/// let allocator = Allocator::new();
///
/// let s1 = "foo";
/// let s2 = "bar";
/// let formatted = format_ident!(&allocator, "{s1}.{s2}");
/// assert_eq!(formatted, "foo.bar");
/// ```
#[macro_export]
macro_rules! format_ident {
    ($alloc:expr, $($arg:tt)*) => {{
        let mut s = $crate::compiler_strings::__internal::ArenaStringBuilder::new_in($alloc);
        ::std::fmt::Write::write_fmt(&mut s, ::std::format_args!($($arg)*)).unwrap();
        $crate::compiler_strings::Ident::from(s)
    }}
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_strings/src/ident/test.rs"]
#[cfg(test)]
mod test;
