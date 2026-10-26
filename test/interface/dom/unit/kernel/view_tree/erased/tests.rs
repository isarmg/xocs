use super::{Erased, ErasedLocal};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

macro_rules! type_checks {
    ($module:ident, $erased:ty) => {
        mod $module {
            use super::*;

            #[test]
            fn matching_types_can_be_read_updated_and_taken() {
                let mut erased = <$erased>::new(String::from("first"));
                assert_eq!(erased.get_ref::<String>(), "first");
                *erased.get_mut::<String>() = String::from("second");
                assert_eq!(erased.into_inner::<String>(), "second");
            }

            #[test]
            #[should_panic(expected = "Erased: type mismatch")]
            fn shared_access_rejects_a_different_type() {
                let erased = <$erased>::new(0u8);
                erased.get_ref::<bool>();
            }

            #[test]
            #[should_panic(expected = "Erased: type mismatch")]
            fn mutable_access_rejects_a_different_type() {
                let mut erased = <$erased>::new(0u8);
                erased.get_mut::<bool>();
            }

            #[test]
            #[should_panic(expected = "Erased: type mismatch")]
            fn consumption_rejects_a_different_type() {
                <$erased>::new(0u8).into_inner::<bool>();
            }

            #[test]
            fn rejecting_a_type_drops_the_original_value_once() {
                struct Tracked(Arc<AtomicUsize>);
                impl Drop for Tracked {
                    fn drop(&mut self) {
                        self.0.fetch_add(1, Ordering::Relaxed);
                    }
                }
                let drops = Arc::new(AtomicUsize::new(0));
                let erased = <$erased>::new(Tracked(Arc::clone(&drops)));
                let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    erased.into_inner::<String>();
                }));
                assert!(rejected.is_err());
                assert_eq!(drops.load(Ordering::Relaxed), 1);
            }
        }
    };
}

type_checks!(send, Erased);
type_checks!(local, ErasedLocal);
