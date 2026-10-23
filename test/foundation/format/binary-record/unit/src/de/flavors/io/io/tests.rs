
            use super::*;

            #[test]
            fn test_pop() {
                let mut reader = IOReader::new(&[0xAA, 0xBB, 0xCC][..], &mut []);

                assert_eq!(reader.pop(), Ok(0xAA));
                assert_eq!(reader.pop(), Ok(0xBB));
                assert_eq!(reader.pop(), Ok(0xCC));
                assert_eq!(reader.pop(), Err(Error::DeserializeUnexpectedEnd));
            }

            #[test]
            fn test_try_take_n() {
                let mut buf = [0; 8];
                let mut reader = IOReader::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE][..], &mut buf);

                assert_eq!(reader.try_take_n(2), Ok(&[0xAA, 0xBB][..]));
                assert_eq!(reader.try_take_n(2), Ok(&[0xCC, 0xDD][..]));
                assert_eq!(reader.try_take_n(2), Err(Error::DeserializeUnexpectedEnd));
            }
        
