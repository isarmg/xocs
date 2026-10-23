use core::sync::atomic::*;

#[cfg(target_has_atomic = "8")]
forward_impl!(AtomicBool => bool);

#[cfg(target_has_atomic = "8")]
forward_impl!(AtomicI8 => i8);

#[cfg(target_has_atomic = "16")]
forward_impl!(AtomicI16 => i16);

#[cfg(target_has_atomic = "32")]
forward_impl!(AtomicI32 => i32);

#[cfg(target_has_atomic = "64")]
forward_impl!(AtomicI64 => i64);

#[cfg(target_has_atomic = "ptr")]
forward_impl!(AtomicIsize => isize);

#[cfg(target_has_atomic = "8")]
forward_impl!(AtomicU8 => u8);

#[cfg(target_has_atomic = "16")]
forward_impl!(AtomicU16 => u16);

#[cfg(target_has_atomic = "32")]
forward_impl!(AtomicU32 => u32);

#[cfg(target_has_atomic = "64")]
forward_impl!(AtomicU64 => u64);

#[cfg(target_has_atomic = "ptr")]
forward_impl!(AtomicUsize => usize);

#[path = "../../../../../../test/foundation/format/json-schema/unit/src/json_schema_impls/atomic/tests.rs"]
#[cfg(test)]
mod tests;
