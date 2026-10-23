macro_rules! nonmax_tests {
    ($nonmax:ident, $primitive:ident) => {
#[cfg(test)]
        mod $primitive {
            use super::*;

            use core::mem::size_of;

            #[test]
            fn construct() {
                let zero = $nonmax::new(0).unwrap();
                assert_eq!(zero.get(), 0);

                let some = $nonmax::new(19).unwrap();
                assert_eq!(some.get(), 19);

                let max = $nonmax::new($primitive::MAX);
                assert_eq!(max, None);
            }

            #[test]
            fn sizes_correct() {
                assert_eq!(size_of::<$primitive>(), size_of::<$nonmax>());
                assert_eq!(size_of::<$nonmax>(), size_of::<Option<$nonmax>>());
            }

            #[test]
            fn convert() {
                use core::convert::TryFrom;
                let zero = $nonmax::try_from(0 as $primitive).unwrap();
                let zero = $primitive::from(zero);
                assert_eq!(zero, 0);

                $nonmax::try_from($primitive::MAX).unwrap_err();
            }

            #[test]
            fn cmp() {
                let zero = $nonmax::new(0).unwrap();
                let one = $nonmax::new(1).unwrap();
                let two = $nonmax::new(2).unwrap();
                assert!(zero < one);
                assert!(one < two);
                assert!(two > one);
                assert!(one > zero);
            }

            #[test]
            fn constants() {
                let zero = $nonmax::ZERO;
                let one = $nonmax::ONE;
                let max = $nonmax::MAX;
                assert_eq!(zero.get(), 0);
                assert_eq!(one.get(), 1);
                assert_eq!(max.get(), $primitive::MAX - 1);
            }

            #[test]
            #[cfg(all())] // to_string
            fn parse() {
                for value in [0, 19, $primitive::MAX - 1].iter().copied() {
                    let string = value.to_string();
                    let nonmax = string.parse::<$nonmax>().unwrap();
                    assert_eq!(nonmax.get(), value);
                }
                $primitive::MAX.to_string().parse::<$nonmax>().unwrap_err();
            }

            #[test]
            #[cfg(all())] // format!
            fn fmt() {
                let zero = $nonmax::new(0).unwrap();
                let some = $nonmax::new(19).unwrap();
                let max1 = $nonmax::new($primitive::MAX - 1).unwrap();
                for value in [zero, some, max1].iter().copied() {
                    assert_eq!(format!("{}", value.get()), format!("{}", value)); // Display
                    assert_eq!(format!("{:?}", value.get()), format!("{:?}", value)); // Debug
                    assert_eq!(format!("{:b}", value.get()), format!("{:b}", value)); // Binary
                    assert_eq!(format!("{:o}", value.get()), format!("{:o}", value)); // Octal
                    assert_eq!(format!("{:x}", value.get()), format!("{:x}", value)); // LowerHex
                    assert_eq!(format!("{:X}", value.get()), format!("{:X}", value)); // UpperHex
                }
            }

            #[test]
            #[cfg(any())]
            fn serde() {
                for &value in [0, 19, $primitive::MAX - 1].iter() {
                    let nonmax_value = $nonmax::new(value).unwrap();
                    let encoded: Vec<u8> = bincode::serialize(&nonmax_value).unwrap();
                    let decoded: $nonmax = bincode::deserialize(&encoded[..]).unwrap();
                    assert_eq!(nonmax_value, decoded);
                }
            }
        }
    };
}
