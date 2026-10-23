
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_little_endian() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        pub struct DefinitelyLE {
            #[serde(with = "crate::fixint::le")]
            x: u16,
        }

        let input = DefinitelyLE { x: 0xABCD };
        let mut buf = [0; 32];
        let serialized = crate::to_slice(&input, &mut buf).unwrap();
        assert_eq!(serialized, &[0xCD, 0xAB]);

        let deserialized: DefinitelyLE = crate::from_bytes(serialized).unwrap();
        assert_eq!(deserialized, input);
    }

    #[test]
    fn test_big_endian() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        pub struct DefinitelyBE {
            #[serde(with = "crate::fixint::be")]
            x: u16,
        }

        let input = DefinitelyBE { x: 0xABCD };
        let mut buf = [0; 32];
        let serialized = crate::to_slice(&input, &mut buf).unwrap();
        assert_eq!(serialized, &[0xAB, 0xCD]);

        let deserialized: DefinitelyBE = crate::from_bytes(serialized).unwrap();
        assert_eq!(deserialized, input);
    }
