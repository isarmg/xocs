#[cfg(test)]
    macro_rules! assert_clean {
        ($e:ident) => {{
            assert!($e.pointers.get_next().is_none());
            assert!($e.pointers.get_prev().is_none());
        }};
    }
