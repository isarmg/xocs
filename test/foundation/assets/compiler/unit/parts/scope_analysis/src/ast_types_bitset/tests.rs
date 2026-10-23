
    use super::*;
    use crate::syntax_tree::AstType;

    #[test]
    fn empty_bitset_has_no_bits_and_contains_empty() {
        let bs = AstTypesBitset::new();
        assert!(!bs.has(AstType::Program));
        let other = AstTypesBitset::new();
        assert!(bs.contains(&other), "Empty bitset should contain empty bitset");
        assert!(!bs.intersects(&other));
    }

    #[test]
    fn intersects_and_contains() {
        let mut a = AstTypesBitset::from_types(&[AstType::Program, AstType::AssignmentPattern]);
        let b = AstTypesBitset::from_types(&[AstType::TSTupleType]);
        assert!(!a.intersects(&b));
        a.set(AstType::TSTupleType);
        assert!(a.intersects(&b));

        let c = AstTypesBitset::from_types(&[AstType::Program]);
        assert!(a.contains(&c));
        assert!(!c.contains(&a));

        // a should contain union of subset bits
        let subset = AstTypesBitset::from_types(&[AstType::Program, AstType::AssignmentPattern]);
        assert!(a.contains(&subset));
        // subset does not contain a (missing TSTupleType)
        assert!(!subset.contains(&a));
    }

    #[test]
    fn contains_empty_is_true() {
        let non_empty = AstTypesBitset::from_types(&[AstType::IdentifierName]);
        let empty = AstTypesBitset::new();
        assert!(non_empty.contains(&empty));
    }

    #[test]
    fn iter_empty() {
        let bs = AstTypesBitset::new();
        assert_eq!(bs.iter().count(), 0);
    }

    #[test]
    fn iter_collects_set_bits_in_order() {
        let types = [AstType::Program, AstType::AssignmentPattern, AstType::TSTupleType];
        let bs = AstTypesBitset::from_types(&types);
        let collected: Vec<AstType> = bs.iter().collect();
        assert_eq!(collected, types);
    }
