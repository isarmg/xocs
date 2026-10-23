
    use crate::syntax_tree::ast::TSAccessibility;

    use super::{ModifierKind, ModifierKinds};

    #[test]
    fn accessibility_none() {
        assert_eq!(ModifierKinds::none().accessibility(), None);
    }

    #[test]
    fn accessibility_non_accessibility_modifiers() {
        for kind in ModifierKind::VARIANTS {
            if matches!(
                kind,
                ModifierKind::Private | ModifierKind::Protected | ModifierKind::Public
            ) {
                continue;
            }
            assert_eq!(ModifierKinds::new([kind]).accessibility(), None, "{kind}");
        }
    }

    #[test]
    fn accessibility_single() {
        assert_eq!(
            ModifierKinds::new([ModifierKind::Private]).accessibility(),
            Some(TSAccessibility::Private),
        );
        assert_eq!(
            ModifierKinds::new([ModifierKind::Protected]).accessibility(),
            Some(TSAccessibility::Protected),
        );
        assert_eq!(
            ModifierKinds::new([ModifierKind::Public]).accessibility(),
            Some(TSAccessibility::Public),
        );
    }

    #[test]
    fn accessibility_with_other_modifiers() {
        let kinds = ModifierKinds::new([ModifierKind::Static, ModifierKind::Public]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Public));

        let kinds = ModifierKinds::new([
            ModifierKind::Async,
            ModifierKind::Protected,
            ModifierKind::Override,
        ]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Protected));
    }

    #[test]
    fn accessibility_multiple_accessibility_modifiers() {
        // Multiple accessibility modifiers is a syntax error, but the function should still
        // return a deterministic result. Priority: Public > Protected > Private.
        let kinds = ModifierKinds::new([ModifierKind::Private, ModifierKind::Protected]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Protected));

        let kinds = ModifierKinds::new([ModifierKind::Private, ModifierKind::Public]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Public));

        let kinds = ModifierKinds::new([ModifierKind::Protected, ModifierKind::Public]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Public));

        let kinds = ModifierKinds::new([
            ModifierKind::Private,
            ModifierKind::Protected,
            ModifierKind::Public,
        ]);
        assert_eq!(kinds.accessibility(), Some(TSAccessibility::Public));
    }
