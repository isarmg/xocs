
    use crate::syntax_arena::Allocator;

    use super::decode_entities;

    #[test]
    fn entity_after_stray_amp() {
        let allocator = Allocator::default();
        let input = "& &amp;";
        let mut acc = None;
        decode_entities(input, &mut acc, input.len(), &allocator);
        assert_eq!(acc.as_ref().unwrap().as_str(), "& &");
    }
