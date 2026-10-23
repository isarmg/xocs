
    use super::*;

    #[cfg(feature = "alloc")]
    use alloc::string::String;

    #[test]
    fn cast() {
        assert_eq!(cast!(0u8, u16), Err(0u8));
        assert_eq!(cast!(1u8, u8), Ok(1u8));
        assert_eq!(cast!(2u8, &'static u8), Err(2u8));
        assert_eq!(cast!(2u8, &u8), Err(2u8)); // 'static is inferred

        static VALUE: u8 = 2u8;
        assert_eq!(cast!(&VALUE, &u8), Ok(&2u8));
        assert_eq!(cast!(&VALUE, &'static u8), Ok(&2u8));
        assert_eq!(cast!(&VALUE, &u16), Err(&2u8));
        assert_eq!(cast!(&VALUE, &i8), Err(&2u8));

        let value = 2u8;

        fn inner<'a>(value: &'a u8) {
            assert_eq!(cast!(value, &u8), Ok(&2u8));
            assert_eq!(cast!(value, &'a u8), Ok(&2u8));
            assert_eq!(cast!(value, &u16), Err(&2u8));
            assert_eq!(cast!(value, &i8), Err(&2u8));
        }

        inner(&value);

        let mut slice = [1u8; 2];

        fn inner2<'a>(value: &'a [u8]) {
            assert_eq!(cast!(value, &[u8]), Ok(&[1, 1][..]));
            assert_eq!(cast!(value, &'a [u8]), Ok(&[1, 1][..]));
            assert_eq!(cast!(value, &'a [u16]), Err(&[1, 1][..]));
            assert_eq!(cast!(value, &'a [i8]), Err(&[1, 1][..]));
        }

        inner2(&slice);

        #[allow(clippy::needless_lifetimes)]
        fn inner3<'a>(value: &'a mut [u8]) {
            assert_eq!(cast!(value, &mut [u8]), Ok(&mut [1, 1][..]));
        }
        inner3(&mut slice);
    }

    #[test]
    fn cast_with_type_inference() {
        let result: Result<u8, u8> = cast!(0u8);
        assert_eq!(result, Ok(0u8));

        let result: Result<u8, u16> = cast!(0u16);
        assert_eq!(result, Err(0u16));
    }

    #[test]
    fn match_type() {
        let v = 42i32;

        assert!(match_type!(v, {
            u32 as _ => false,
            i32 as _ => true,
            _ => false,
        }));
    }

    #[test]
    fn cast_lifetime_free_unsized_ref() {
        fn can_cast<T>(value: &[T]) -> bool {
            cast!(value, &[u8]).is_ok()
        }

        let value = 42i32;
        assert!(can_cast(&[1_u8, 2, 3, 4]));
        assert!(!can_cast(&[1_i8, 2, 3, 4]));
        assert!(!can_cast(&[&value, &value]));
    }

    #[test]
    fn cast_lifetime_free_unsized_mut() {
        fn can_cast<T>(value: &mut [T]) -> bool {
            cast!(value, &mut [u8]).is_ok()
        }

        let value = 42i32;
        assert!(can_cast(&mut [1_u8, 2, 3, 4]));
        assert!(!can_cast(&mut [1_i8, 2, 3, 4]));
        assert!(!can_cast(&mut [&value, &value]));
    }

    macro_rules! test_lifetime_free_cast {
        () => {};

        (
            $(#[$meta:meta])*
            for $TARGET:ty as $name:ident {
                $(
                    $value:expr => $matches:pat $(if $guard:expr)?,
                )+
            }
            $($tail:tt)*
        ) => {
            paste::paste! {
                $(#[$meta])*
                #[test]
                #[allow(non_snake_case)]
                fn [<cast_lifetime_free_ $name>]() {
                    fn do_cast<T>(value: T) -> Result<$TARGET, T> {
                        cast!(value, $TARGET)
                    }

                    $(
                        assert!(match do_cast($value) {
                            $matches $(if $guard)* => true,
                            _ => false,
                        });
                    )*
                }

                $(#[$meta])*
                #[test]
                #[allow(non_snake_case)]
                fn [<cast_lifetime_free_ref_ $name>]() {
                    fn do_cast<T>(value: &T) -> Result<&$TARGET, &T> {
                        cast!(value, &$TARGET)
                    }

                    $(
                        assert!(match do_cast(&$value).map(|t| t.clone()).map_err(|e| e.clone()) {
                            $matches $(if $guard)* => true,
                            _ => false,
                        });
                    )*
                }

                $(#[$meta])*
                #[test]
                #[allow(non_snake_case)]
                fn [<cast_lifetime_free_mut_ $name>]() {
                    fn do_cast<T>(value: &mut T) -> Result<&mut $TARGET, &mut T> {
                        cast!(value, &mut $TARGET)
                    }

                    $(
                        assert!(match do_cast(&mut $value).map(|t| t.clone()).map_err(|e| e.clone()) {
                            $matches $(if $guard)* => true,
                            _ => false,
                        });
                    )*
                }
            }

            test_lifetime_free_cast! {
                $($tail)*
            }
        };

        (
            $(#[$meta:meta])*
            for $TARGET:ty {
                $(
                    $value:expr => $matches:pat $(if $guard:expr)?,
                )+
            }
            $($tail:tt)*
        ) => {
            paste::paste! {
                $(#[$meta])*
                #[test]
                #[allow(non_snake_case)]
                fn [<cast_lifetime_free_ $TARGET>]() {
                    fn do_cast<T>(value: T) -> Result<$TARGET, T> {
                        cast!(value, $TARGET)
                    }

                    $(
                        assert!(match do_cast($value) {
                            $matches $(if $guard)* => true,
                            _ => false,
                        });
                    )*
                }

                $(#[$meta])*
                #[test]
                #[allow(non_snake_case)]
                fn [<cast_lifetime_free_ref_ $TARGET>]() {
                    fn do_cast<T>(value: &T) -> Result<&$TARGET, &T> {
                        cast!(value, &$TARGET)
                    }

                    $(
                        assert!(match do_cast(&$value).map(|t| t.clone()).map_err(|e| e.clone()) {
                            $matches $(if $guard)* => true,
                            _ => false,
                        });
                    )*
                }

                $(#[$meta])*
                #[test]
                #[allow(non_snake_case)]
                fn [<cast_lifetime_free_mut_ $TARGET>]() {
                    fn do_cast<T>(value: &mut T) -> Result<&mut $TARGET, &mut T> {
                        cast!(value, &mut $TARGET)
                    }

                    $(
                        assert!(match do_cast(&mut $value).map(|t| t.clone()).map_err(|e| e.clone()) {
                            $matches $(if $guard)* => true,
                            _ => false,
                        });
                    )*
                }
            }

            test_lifetime_free_cast! {
                $($tail)*
            }
        };
    }

    test_lifetime_free_cast! {
        for bool {
            0u8 => Err(_),
            true => Ok(true),
        }

        for u8 {
            0u8 => Ok(0u8),
            1u16 => Err(1u16),
            42u8 => Ok(42u8),
        }

        for f32 {
            3.2f32 => Ok(v) if v == 3.2,
            3.2f64 => Err(v) if v == 3.2f64,
        }

        #[cfg(feature = "alloc")]
        for String {
            String::from("hello world") => Ok(ref v) if v.as_str() == "hello world",
            "hello world" => Err("hello world"),
        }

        for Option<u8> as Option_u8 {
            0u8 => Err(0u8),
            Some(42u8) => Ok(Some(42u8)),
        }

        // See https://github.com/rust-lang/rust/issues/127286 for details.
        for (u8, u8) as TupleU8U8 {
            (1u8, 2u8) => Ok((1u8, 2u8)),
            1u8 => Err(1u8),
        }
        for (bool, u16) as TupleBoolU16 {
            (false, 2u16) => Ok((false, 2u16)),
            true => Err(true),
        }
    }
