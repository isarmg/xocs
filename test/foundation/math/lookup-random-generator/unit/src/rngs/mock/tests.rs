
    #[test]
    #[cfg(feature = "serde1")]
    fn test_serialization_step_rng() {
        use super::StepRng;

        let some_rng = StepRng::new(42, 7);
        let de_some_rng: StepRng =
            bincode::deserialize(&bincode::serialize(&some_rng).unwrap()).unwrap();
        assert_eq!(some_rng.v, de_some_rng.v);
        assert_eq!(some_rng.a, de_some_rng.a);

    }
