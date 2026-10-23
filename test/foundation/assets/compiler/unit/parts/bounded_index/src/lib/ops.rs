
    use super::*;

    #[test]
    fn bitand_unsigned() {
        for left in 0..=u8::MAX {
            let nmleft = NonMaxU8::new(left);
            for right in 0..=u8::MAX {
                let nmright = NonMaxU8::new(right);
                let vanilla = left & right;

                if let (Some(nmleft), Some(nmright)) = (nmleft, nmright) {
                    assert_eq!(vanilla, (nmleft & nmright).get());
                }
                if let Some(nmleft) = nmleft {
                    assert_eq!(vanilla, (nmleft & right).get());
                }
                if let Some(nmright) = nmright {
                    assert_eq!(vanilla, (left & nmright).get());
                }
            }
        }
    }

    #[test]
    fn bitand_signed() {
        for left in i8::MIN..=i8::MAX {
            let nmleft = NonMaxI8::new(left);
            for right in i8::MIN..=i8::MAX {
                let nmright = NonMaxI8::new(right);
                let vanilla = left & right;
                if let (Some(nmleft), Some(nmright)) = (nmleft, nmright) {
                    assert_eq!(vanilla, (nmleft & nmright).get());
                }
            }
        }
    }
