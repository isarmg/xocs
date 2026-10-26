use crate::erased_value::ErasedBox;

fn check(id_1: &std::any::TypeId, id_2: &std::any::TypeId) {
    if id_1 != id_2 {
        panic!("Erased: type mismatch")
    }
}

macro_rules! erased {
    ([$($new_t_params:tt)*], $name:ident) => {
        /// A type-erased item. This is slightly more efficient than using `Box<dyn Any (+ Send)>`.
        ///
        /// Retrieval checks the requested type before accessing the erased allocation.
        pub struct $name {
            type_id: std::any::TypeId,
            value: Option<ErasedBox>,
            drop: fn(ErasedBox),
        }

        impl $name {
            /// Create a new type-erased item.
            pub fn new<T: $($new_t_params)*>(item: T) -> Self {
                Self {
                    type_id: std::any::TypeId::of::<T>(),
                    value: Some(ErasedBox::new(Box::new(item))),
                    drop: |value| {
                        // SAFETY: This drop function is paired with the allocation
                        // created from Box<T> above.
                        drop(unsafe { value.into_inner::<T>() });
                    },
                }
            }

            /// Get a reference to the inner value.
            ///
            /// # Panics
            /// Panics if `T` differs from the stored type.
            pub fn get_ref<T: 'static>(&self) -> &T {
                check(&self.type_id, &std::any::TypeId::of::<T>());
                // SAFETY: The TypeId check proves T matches the allocation; the
                // shared borrow keeps that allocation alive and immutable.
                unsafe { self.value.as_ref().expect("invariant: an unconsumed erased value is present").get_ref::<T>() }
            }

            /// Get a mutable reference to the inner value.
            ///
            /// # Panics
            /// Panics if `T` differs from the stored type.
            pub fn get_mut<T: 'static>(&mut self) -> &mut T {
                check(&self.type_id, &std::any::TypeId::of::<T>());
                // SAFETY: The TypeId check proves T matches the allocation; the
                // exclusive borrow prevents aliasing while T is accessed.
                unsafe { self.value.as_mut().expect("invariant: an unconsumed erased value is present").get_mut::<T>() }
            }

            /// Consume the item and return the inner value.
            ///
            /// # Panics
            /// Panics if `T` differs from the stored type.
            pub fn into_inner<T: 'static>(mut self) -> T {
                check(&self.type_id, &std::any::TypeId::of::<T>());
                // SAFETY: The TypeId check proves T matches the allocation; taking
                // its box transfers the sole owner and prevents a second drop.
                *unsafe { self.value.take().expect("invariant: an unconsumed erased value is present").into_inner::<T>() }
            }
        }

        /// If into_inner() wasn't called, the value would leak and destructors wouldn't run, this prevents that from happening.
        impl Drop for $name {
            fn drop(&mut self) {
                if let Some(value) = self.value.take() {
                    (self.drop)(value);
                }
            }
        }
    };

}

erased!([Send + 'static], Erased);
erased!(['static], ErasedLocal);

/// SAFETY: `Erased::new` ensures that `T` is `Send` and `'static`.
unsafe impl Send for Erased {}

#[path = "../../../../../../test/interface/dom/unit/kernel/view_tree/erased/tests.rs"]
#[cfg(test)]
mod tests;
