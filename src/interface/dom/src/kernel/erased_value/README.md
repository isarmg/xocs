# Lifetime-preserving type erasure

This module erases the type of a reference or box while retaining its lifetime. These erased references can then be **unsafely** converted back to their original type.
This can be used to store heterogeneous references in a Vec or another datastructure. This is often useful with references into an arena.

The following types are provided:

| type                 | description                                        |
|----------------------|----------------------------------------------------|
| [`Erased<'a>`](crate::erased_value::Erased)    | An erased reference to a value `&'a T`             |
| [`ErasedMut<'a>`](crate::erased_value::ErasedMut) | An erased mutable reference to a value `&'a mut T` |
| [`ErasedBox`](crate::erased_value::ErasedBox)     | An erased box `Box<T>`                              |

## Example

```rust
use blog_dom::erased_value::Erased;

let mut vec: Vec<Erased<'_>> = Vec::new();
vec.push((&5u64).into());
vec.push((&"Hello World").into());

// SAFETY: Above we insered a `u64` into an empty vec, therefore converting the element back to a `u64` is sound.
assert_eq!(unsafe { *vec[0].get::<u64>() }, 5);
// SAFETY: Above we insered a `&'static str` into a vec containing one element, therefore converting the element back to a `&'static str` is sound.
assert_eq!(unsafe { *vec[1].get::<&'static str>() }, "Hello World");
```

## Comparison with `dyn Any`

This module provides similar functionality to the `downcast_ref` and `downcast_mut` methods on [`dyn Any`](std::any::Any).
The differences are:
- Downcast is *checked*, it stores the type id of the type that was stored and checks whether the generic type provided matches upon retrieval.
  The types defined in this module are *unchecked*, and have an unsafe precondition that the type matches.
- Since no type information is stored, a `Erased<'a>` is only one pointer wide, half the size of `&'a dyn Any`
- `dyn Any` can currently only be used on types with a static lifetime. Erased can be used on any type.
