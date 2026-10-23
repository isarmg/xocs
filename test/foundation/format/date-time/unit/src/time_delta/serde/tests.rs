
        use super::{super::MAX, TimeDelta};

        #[test]
        fn test_serde() {
            let duration = TimeDelta::new(123, 456).unwrap();
            assert_eq!(
                serde_json::from_value::<TimeDelta>(serde_json::to_value(duration).unwrap())
                    .unwrap(),
                duration
            );
        }

        #[test]
        #[should_panic(expected = "TimeDelta out of bounds")]
        fn test_serde_oob_panic() {
            let _ =
                serde_json::from_value::<TimeDelta>(serde_json::json!([MAX.secs + 1, 0])).unwrap();
        }
    
