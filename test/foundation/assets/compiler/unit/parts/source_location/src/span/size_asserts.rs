
    use std::mem::{align_of, size_of};

    use super::Span;

    const _: () = assert!(size_of::<Span>() == 8);

    #[cfg(target_pointer_width = "64")]
    const _: () = assert!(align_of::<Span>() == 8);

    #[cfg(not(target_pointer_width = "64"))]
    const _: () = assert!(align_of::<Span>() == 4);
