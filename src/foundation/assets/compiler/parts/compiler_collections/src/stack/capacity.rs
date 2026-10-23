use std::mem::{align_of, size_of};

/// Trait for defining maximum and default capacity of stacks.
///
/// `T` is the type that the stack contains.
///
/// `MAX_CAPACITY` and `MAX_CAPACITY_BYTES` being calculated correctly is required for soundness
/// of stack types.
pub trait StackCapacity<T> {
    /// Maximum capacity of stack.
    ///
    /// This is guaranteed to be a legal size for a stack of `T`s, without exceeding Rust's
    /// allocation size limits.
    ///
    /// From [`std::alloc::Layout`]'s docs:
    /// > size, when rounded up to the nearest multiple of align, must not overflow `isize`
    /// > (i.e., the rounded value must be less than or equal to `isize::MAX`).
    const MAX_CAPACITY: usize = {
        // This assertion is not needed as next line will cause a compile failure anyway
        // if `size_of::<T>() == 0`, due to division by zero.
        // But keep it anyway as soundness depends on it.
        assert!(size_of::<T>() > 0, "Zero sized types are not supported");
        // As it's always true that `size_of::<T>() >= align_of::<T>()` and `/` rounds down,
        // this fulfills `Layout`'s alignment requirement
        let max_capacity = isize::MAX as usize / size_of::<T>();
        assert!(max_capacity > 0);
        max_capacity
    };

    /// Maximum capacity of stack in bytes
    const MAX_CAPACITY_BYTES: usize = {
        let capacity_bytes = Self::MAX_CAPACITY * size_of::<T>();
        // Just double-checking `Layout`'s alignment requirement is fulfilled
        assert!(capacity_bytes <= isize::MAX as usize + 1 - align_of::<T>());
        capacity_bytes
    };

    /// Default capacity of stack.
    ///
    /// Same defaults as [`std::vec::Vec`] uses, except 16 bytes for types with size 1 (instead of 8).
    /// Allocators will usually allocate minimum 16 bytes anyway.
    const DEFAULT_CAPACITY: usize = {
        // It's impossible for this to exceed `MAX_CAPACITY` because `size_of::<T>() >= align_of::<T>()`
        match size_of::<T>() {
            1 => 16,
            size if size <= 1024 => 4,
            _ => 1,
        }
    };

    /// Default capacity of stack in bytes
    const DEFAULT_CAPACITY_BYTES: usize = Self::DEFAULT_CAPACITY * size_of::<T>();
}

#[path = "../../../../../../../../test/foundation/assets/compiler/unit/parts/compiler_collections/src/stack/capacity/tests.rs"]
#[cfg(test)]
#[expect(clippy::assertions_on_constants)]
mod tests;
