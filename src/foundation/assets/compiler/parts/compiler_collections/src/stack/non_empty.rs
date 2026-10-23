use std::{
    error::Error,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::{StackCapacity, StackCommon};

/// A stack which can never be empty.
///
/// [`NonEmptyStack`] is created initially with 1 entry, and [`pop`] does not allow removing it
/// (though that initial entry can be mutated with [`last_mut`]).
///
/// The fact that the stack is never empty makes all operations except [`pop`] infallible.
/// [`last`] and [`last_mut`] are branchless.
///
/// The trade-off is that you cannot create a [`NonEmptyStack`] without allocating,
/// and you must create an initial value for the "dummy" initial entry.
/// If that is not a good trade-off for your use case, prefer [`Stack`], which can be empty.
///
/// [`NonEmptyStack`] is usually a better choice than [`Stack`], unless either:
///
/// 1. The stack will likely never have anything pushed to it.
///    [`NonEmptyStack::new`] always allocates, whereas [`Stack::new`] does not.
///    So if stack usually starts empty and remains empty, [`Stack`] will avoid an allocation.
///    This is the same as how [`Vec`] does not allocate until you push a value into it.
///
/// 2. The type the stack holds is large or expensive to construct, so there's a high cost in having to
///    create an initial dummy value (which [`NonEmptyStack`] requires, but [`Stack`] doesn't).
///
/// [`SparseStack`] may be preferable if the type you're storing is an `Option`.
///
/// To simplify implementation, zero size types are not supported (e.g. `NonEmptyStack<()>`).
///
/// ## Design
/// Designed for maximally efficient [`push`], [`pop`], and reading/writing the last value on stack
/// ([`last`] / [`last_mut`]).
///
/// The alternative would likely be to use a [`Vec`]. But `Vec` is optimized for indexing into at
/// arbitrary positions, not for `push` and `pop`. `Vec` stores `len` and `capacity` as integers,
/// so requires pointer maths on every operation: `let entry_ptr = base_ptr + index * size_of::<T>();`.
///
/// In comparison, [`NonEmptyStack`] contains a `cursor` pointer, which always points to last entry
/// on stack, so it can be read/written with a minimum of operations.
///
/// This design is similar to [`std`'s slice iterators].
///
/// Comparison to [`Vec`]:
/// * [`last`] and [`last_mut`] are 1 instruction, instead of `Vec`'s 4.
/// * [`pop`] is 1 instruction shorter than `Vec`'s equivalent.
/// * [`push`] is 1 instruction shorter than `Vec`'s equivalent, and uses 1 less register.
///
/// ### Possible alternative designs
/// 1. `cursor` could point to *after* last entry, rather than *to* it. This has advantage that [`pop`]
///    uses 1 less register, but disadvantage that [`last`] and [`last_mut`] are 2 instructions, not 1.
///    <https://godbolt.org/z/xnx7YP5de>
///
/// 2. Stack could grow downwards, like `bumpalo` does. This would probably make [`pop`] use
///    1 less register, but at the cost that: (a) the stack can never grow in place, which would incur
///    more memory copies when the stack grows, and (b) [`as_slice`] would have the entries in
///    reverse order.
///
/// [`push`]: NonEmptyStack::push
/// [`pop`]: NonEmptyStack::pop
/// [`last`]: NonEmptyStack::last
/// [`last_mut`]: NonEmptyStack::last_mut
/// [`as_slice`]: NonEmptyStack::as_slice
/// [`Stack`]: super::Stack
/// [`Stack::new`]: super::Stack::new
/// [`SparseStack`]: super::SparseStack
/// [`std`'s slice iterators]: std::slice::Iter
pub struct NonEmptyStack<T> {
    /// Pointer to last entry on stack.
    /// Points *to* last entry, not *after* last entry.
    cursor: NonNull<T>,
    /// Pointer to start of allocation (first entry)
    start: NonNull<T>,
    /// Pointer to end of allocation
    end: NonNull<T>,
    /// Inform compiler that `NonEmptyStack<T>` owns `T`s
    _marker: PhantomData<T>,
}

impl<T> StackCapacity<T> for NonEmptyStack<T> {}

impl<T> StackCommon<T> for NonEmptyStack<T> {
    #[inline]
    fn start(&self) -> NonNull<T> {
        self.start
    }

    #[inline]
    fn end(&self) -> NonNull<T> {
        self.end
    }

    #[inline]
    fn cursor(&self) -> NonNull<T> {
        self.cursor
    }

    #[inline]
    fn set_start(&mut self, start: NonNull<T>) {
        self.start = start;
    }

    #[inline]
    fn set_end(&mut self, end: NonNull<T>) {
        self.end = end;
    }

    #[inline]
    fn set_cursor(&mut self, cursor: NonNull<T>) {
        self.cursor = cursor;
    }

    #[inline]
    fn len(&self) -> usize {
        // SAFETY: `self.start` and `self.cursor` are both derived from same pointer.
        // `self.cursor` is always >= `self.start`.
        // Distance between pointers is always a multiple of `size_of::<T>()`.
        let offset = unsafe { self.cursor_offset() };

        // When stack has 1 entry, `start - cursor == 0`, so add 1 to get number of entries.
        // SAFETY: Capacity cannot exceed `Self::MAX_CAPACITY`, which is `<= isize::MAX`,
        // and offset can't exceed capacity, so `+ 1` cannot wrap around.
        // `unchecked_add(1)` instead of just `+ 1` to hint to compiler that return value can never be zero.
        unsafe { offset.unchecked_add(1) }
    }
}

impl<T> NonEmptyStack<T> {
    /// Maximum capacity.
    ///
    /// Effectively unlimited on 64-bit systems.
    pub const MAX_CAPACITY: usize = <Self as StackCapacity<T>>::MAX_CAPACITY;

    /// Create new [`NonEmptyStack`] with default pre-allocated capacity, and initial value `initial_value`.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    #[inline]
    pub fn new(initial_value: T) -> Self {
        // SAFETY: `DEFAULT_CAPACITY_BYTES` satisfies requirements
        unsafe {
            Self::new_with_capacity_bytes_unchecked(Self::DEFAULT_CAPACITY_BYTES, initial_value)
        }
    }

    /// Create new [`NonEmptyStack`] with pre-allocated capacity for `capacity` entries,
    /// and initial value `initial_value`.
    ///
    /// `capacity` cannot be 0.
    ///
    /// # Panics
    /// Panics if any of these requirements are not satisfied:
    /// * `T` must not be a zero-sized type.
    /// * `capacity` must not be 0.
    /// * `capacity` must not exceed [`Self::MAX_CAPACITY`].
    #[inline]
    pub fn with_capacity(capacity: usize, initial_value: T) -> Self {
        assert!(capacity > 0, "`capacity` cannot be zero");
        assert!(capacity <= Self::MAX_CAPACITY, "`capacity` must not exceed `Self::MAX_CAPACITY`");
        // SAFETY: Assertions above ensure `capacity` satisfies requirements
        unsafe { Self::with_capacity_unchecked(capacity, initial_value) }
    }

    /// Create new [`NonEmptyStack`] with pre-allocated capacity for `capacity` entries,
    /// and initial value `initial_value`, without checks.
    ///
    /// `capacity` cannot be 0.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    ///
    /// # SAFETY
    ///
    /// * `capacity` must not be 0.
    /// * `capacity` must not exceed [`Self::MAX_CAPACITY`].
    #[inline]
    pub unsafe fn with_capacity_unchecked(capacity: usize, initial_value: T) -> Self {
        debug_assert!(capacity > 0);
        debug_assert!(capacity <= Self::MAX_CAPACITY);

        // Cannot overflow if `capacity <= MAX_CAPACITY`
        let capacity_bytes = capacity * size_of::<T>();

        // SAFETY: Safety invariants which caller must satisfy guarantee that `capacity_bytes`
        // satisfies requirements
        unsafe { Self::new_with_capacity_bytes_unchecked(capacity_bytes, initial_value) }
    }

    /// Create new [`NonEmptyStack`] with provided capacity in bytes, and initial value `initial_value`,
    /// without checks.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    ///
    /// # SAFETY
    /// * `capacity_bytes` must not be 0.
    /// * `capacity_bytes` must be a multiple of `size_of::<T>()`.
    /// * `capacity_bytes` must not exceed [`Self::MAX_CAPACITY_BYTES`].
    #[inline]
    unsafe fn new_with_capacity_bytes_unchecked(capacity_bytes: usize, initial_value: T) -> Self {
        // ZSTs are not supported for simplicity
        const { assert!(size_of::<T>() > 0, "Zero sized types are not supported") };

        // SAFETY: Caller guarantees `capacity_bytes` satisfies requirements
        let (start, end) = unsafe { Self::allocate(capacity_bytes) };

        // Write initial value to start of allocation.
        // SAFETY: Allocation was created with alignment of `T`, and with capacity for at least 1 entry,
        // so `start` is valid for writing a `T`.
        unsafe { start.as_ptr().write(initial_value) };

        // `cursor` is positioned at start i.e. pointing at initial value
        Self { cursor: start, start, end, _marker: PhantomData }
    }

    /// Get reference to first value on stack.
    #[inline]
    pub fn first(&self) -> &T {
        // SAFETY: All methods ensure stack is never empty, so `start` always points to
        // a valid initialized `T`. `start` is always aligned for `T`.
        unsafe { self.start.as_ref() }
    }

    /// Get mutable reference to first value on stack.
    #[inline]
    pub fn first_mut(&mut self) -> &mut T {
        // SAFETY: All methods ensure stack is never empty, so `start` always points to
        // a valid initialized `T`. `start` is always aligned for `T`.
        unsafe { self.start.as_mut() }
    }

    /// Get reference to last value on stack.
    #[inline]
    pub fn last(&self) -> &T {
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and points to a valid initialized `T`
        unsafe { self.cursor.as_ref() }
    }

    /// Get mutable reference to last value on stack.
    #[inline]
    pub fn last_mut(&mut self) -> &mut T {
        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and points to a valid initialized `T`
        unsafe { self.cursor.as_mut() }
    }

    /// Push value to stack.
    ///
    /// # Panics
    /// Panics if stack is already filled to maximum capacity.
    #[inline]
    pub fn push(&mut self, value: T) {
        // SAFETY: Stack is never empty and `self.cursor` is always less than `self.end`, which is end
        // of allocation. So advancing by a `T` cannot be out of bounds.
        // The distance between `self.cursor` and `self.end` is always a multiple of `size_of::<T>()`,
        // so `==` check is sufficient to detect when full to capacity.
        let new_cursor = unsafe { self.cursor.add(1) };
        if new_cursor == self.end {
            // Needs to grow
            // SAFETY: Stack is full to capacity
            unsafe { self.push_slow(value) };
        } else {
            // Capacity for at least 1 more entry
            // SAFETY: We checked there is capacity for 1 more entry, so `self.cursor` is in bounds.
            // `self.cursor` was aligned for `T`, and we added `size_of::<T>()` to pointer.
            // `size_of::<T>()` is always a multiple of `T`'s alignment, so `self.cursor` must still be
            // aligned for `T`.
            unsafe { new_cursor.as_ptr().write(value) };
            self.cursor = new_cursor;
        }
    }

    /// Push value to stack when stack is full to capacity.
    ///
    /// This is the slow branch of `push`, which is rarely taken, so marked as `#[cold]` and
    /// `#[inline(never)]` to make `push` as small as possible, so it can be inlined.
    ///
    /// # Panics
    /// Panics if stack is already at maximum capacity.
    ///
    /// # SAFETY
    /// Stack must be full to capacity. i.e. `self.cursor.add(1) == self.end`.
    #[cold]
    #[inline(never)]
    unsafe fn push_slow(&mut self, value: T) {
        // Grow allocation.
        // SAFETY: Stack is always allocated.
        unsafe { self.grow() };

        // Write value.
        // SAFETY: We just allocated additional capacity, so `self.cursor` is in bounds.
        // `self.cursor` is aligned for `T`.
        unsafe { self.cursor.as_ptr().write(value) };
    }

    /// Pop value from stack.
    ///
    /// # Panics
    /// Panics if the stack has only 1 entry on it.
    #[inline]
    pub fn pop(&mut self) -> T {
        // Panic if trying to remove last entry from stack.
        //
        // Putting the panic in an `#[inline(never)]` + `#[cold]` function removes a 6-byte `lea`
        // instruction vs `assert!(self.cursor != self.start, "Cannot pop all entries")`.
        // This reduces this function on x86_64 from 32 bytes to 26 bytes.
        // This function is commonly used, and we want it to be inlined, so every byte counts.
        // https://godbolt.org/z/5587z99rM
        #[inline(never)]
        #[cold]
        fn error() -> ! {
            panic!("Cannot pop all entries");
        }

        if self.cursor == self.start {
            error();
        }

        // SAFETY: Assertion above ensures stack has at least 2 entries
        unsafe { self.pop_unchecked() }
    }

    /// Pop value from stack, without checking that stack isn't empty.
    ///
    /// # SAFETY
    ///
    /// * Stack must have at least 2 entries, so that after pop, it still has at least 1.
    #[inline]
    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(self.cursor > self.start);
        debug_assert!(self.cursor < self.end);

        // SAFETY: All methods ensure `self.cursor` is always in bounds, is aligned for `T`,
        // and points to a valid initialized `T`
        let value = unsafe { self.cursor.read() };

        // SAFETY: Caller guarantees there's at least 2 entries on stack, so subtracting 1
        // cannot be out of bounds
        unsafe { self.cursor = self.cursor.sub(1) };

        value
    }

    /// Get number of values on stack.
    ///
    /// Number of entries is always at least 1. Stack is never empty.
    #[inline]
    pub fn len(&self) -> usize {
        <Self as StackCommon<T>>::len(self)
    }

    /// Get if stack is empty.
    ///
    /// This method is pointless, as the stack is never empty.
    /// Only present to override the default method from `[T]::is_empty` which is inherited via `Deref`.
    ///
    /// Using this method is a compile-time error, because using it is certainly a logic error.
    ///
    /// Use [`is_exhausted`] instead.
    ///
    /// [`is_exhausted`]: Self::is_exhausted
    #[expect(clippy::unused_self)]
    #[inline]
    pub fn is_empty(&self) -> bool {
        const {
            panic!("`NonEmptyStack` is never empty. Use `is_exhausted` instead.");
        }
    }

    /// Get if stack is back in its initial state.
    /// i.e. Every `push` has been followed by a corresponding `pop`.
    ///
    /// [`NonEmptyStack`] is never empty, so this is the closest thing to `is_empty` that makes sense.
    ///
    /// If this method returns `true`, the stack only contains the initial element,
    /// and calling [`pop`] will panic.
    ///
    /// Equivalent to `stack.len() == 1`.
    ///
    /// [`pop`]: Self::pop
    #[inline]
    pub fn is_exhausted(&self) -> bool {
        self.cursor == self.start
    }

    /// Get capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        <Self as StackCommon<T>>::capacity(self)
    }

    /// Get contents of stack as a slice `&[T]`.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        <Self as StackCommon<T>>::as_slice(self)
    }

    /// Get contents of stack as a mutable slice `&mut [T]`.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        <Self as StackCommon<T>>::as_mut_slice(self)
    }

    /// Convert [`NonEmptyStack`] into a [`Vec`].
    ///
    /// This is a cheap conversion. The `Vec` takes over ownership of the `NonEmptyStack`'s allocation.
    /// No allocation occurs, and no data is copied.
    #[inline] // This method is just a few arithmetic operations
    pub fn into_vec(self) -> Vec<T> {
        debug_assert!(self.len() <= self.capacity());

        // Wrap `self` in `ManuallyDrop` to prevent it from being dropped
        let stack = ManuallyDrop::new(self);

        // SAFETY:
        // T is not zero-sized. It's impossible to create a `Stack<T>` with a ZST `T`.
        //
        // If the `NonEmptyStack` has allocated:
        // * `start` is the pointer is was allocated with.
        // * `start` is aligned for `T`.
        // * `size_of::<T>() * capacity` is equal to the size of the allocation which backs the `NonEmptyStack`.
        // * This is the size the allocation was originally allocated with.
        // * `len <= capacity`
        // * The first `len` values are properly initialized values of type `T`.
        //
        // If the `NonEmptyStack` has not allocated
        // * `start` is a dangling pointer, aligned for `T`.
        //
        // In short, `NonEmptyStack` manages it's allocation exactly the same way as `Vec`,
        // so it's safe to convert between the two.
        unsafe { Vec::from_raw_parts(stack.start.as_ptr(), stack.len(), stack.capacity()) }
    }
}

impl<T> Drop for NonEmptyStack<T> {
    fn drop(&mut self) {
        // Drop contents.
        // SAFETY: Stack is always allocated, and contains `self.len()` initialized entries,
        // starting at `self.start`.
        unsafe { self.drop_contents() };

        // Drop the memory
        // SAFETY: Stack is always allocated.
        unsafe { self.deallocate() };
    }
}

impl<T> Deref for NonEmptyStack<T> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for NonEmptyStack<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Default> Default for NonEmptyStack<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Error type for [`TryFrom<Vec>`].
pub struct TryFromVecError;

impl Error for TryFromVecError {}

impl Display for TryFromVecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Cannot create a `NonEmptyStack` from an empty `Vec`")
    }
}

impl Debug for TryFromVecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<T> TryFrom<Vec<T>> for NonEmptyStack<T> {
    type Error = TryFromVecError;

    /// Try to convert a [`Vec`] into a [`NonEmptyStack`].
    ///
    /// This is a cheap conversion. The `NonEmptyStack` takes over takes ownership of the `Vec`'s allocation.
    /// No allocation occurs, and no data is copied.
    ///
    /// # Errors
    /// Returns `Err` if the `Vec` is empty.
    #[inline] // This method is just a few arithmetic operations
    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        // ZSTs are not supported for simplicity
        const { assert!(size_of::<T>() > 0, "Zero sized types are not supported") };

        if vec.is_empty() {
            return Err(TryFromVecError);
        }

        // Wrap `vec` in `ManuallyDrop` to prevent it from being dropped
        let mut vec = ManuallyDrop::new(vec);

        // SAFETY: `Vec`'s pointer is always non-null, even when it's not allocated.
        // In this case we know it *has* allocated, because it's not empty, and `T` is not zero-sized.
        let ptr = unsafe { NonNull::new_unchecked(vec.as_mut_ptr()) };

        // SAFETY:
        // We ensured that `T` is not zero-sized above, and that `vec` is not empty.
        //
        // `ptr + len - 1` for a non-empty `Vec` is always in bounds of the `Vec`'s allocation,
        // and `ptr + capacity` is always the end of the `Vec`'s allocation.
        // Therefore these `add` operations are within bounds.
        //
        // The `NonEmptyStack` takes ownership of the allocation, and will deallocate it correctly when dropped.
        let stack = unsafe {
            Self {
                cursor: ptr.add(vec.len() - 1),
                start: ptr,
                end: ptr.add(vec.capacity()),
                _marker: PhantomData,
            }
        };

        Ok(stack)
    }
}

impl<T> From<NonEmptyStack<T>> for Vec<T> {
    /// Convert a [`NonEmptyStack`] into a [`Vec`].
    ///
    /// This is a cheap conversion. The `Vec` takes over ownership of the `NonEmptyStack`'s allocation.
    /// No allocation occurs, and no data is copied.
    #[inline]
    fn from(stack: NonEmptyStack<T>) -> Self {
        stack.into_vec()
    }
}

impl<T> FromIterator<T> for NonEmptyStack<T> {
    /// Create a [`NonEmptyStack`] from an iterator.
    ///
    /// # Panics
    /// Panics if the iterator is empty.
    #[inline] // It mostly just delegates to `Vec::from_iter`
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        // Collect into a `Vec` first, because `Vec` has specialized implementations of `from_iter`
        // for various kinds of iterators, which we cannot replicate in stable Rust
        let vec = Vec::from_iter(iter);
        Self::try_from(vec)
            .unwrap_or_else(|_| panic!("Cannot create a `NonEmptyStack` from an empty iterator"))
    }
}

// SAFETY: `NonEmptyStack<T>` can be `Send` / `Sync` if `T` is `Send` / `Sync`.
// It does not use interior mutability, and is essentially the same as `Vec<T>` in this respect,
// which implements `Send` / `Sync` in the same way.
unsafe impl<T: Send> Send for NonEmptyStack<T> {}
// SAFETY: See above.
unsafe impl<T: Sync> Sync for NonEmptyStack<T> {}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_collections/src/stack/non_empty/tests.rs"]
#[cfg(test)]
mod tests;
