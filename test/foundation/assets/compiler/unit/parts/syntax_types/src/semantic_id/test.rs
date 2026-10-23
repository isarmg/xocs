
    use std::cell::Cell;

    use crate::syntax_arena::CloneInSemanticIds;

    use crate::syntax_types::node::NodeId;

    use super::SemanticId;

    // `NonMaxU32` cannot represent `u32::MAX`, so `u32::MAX - 1` is the largest ID we can test
    const INPUTS: [u32; 11] = [
        0,
        1,
        2,
        42,
        1000,
        u32::MAX / 2 - 1,
        u32::MAX / 2,
        u32::MAX / 2 + 1,
        u32::MAX - 1000,
        u32::MAX - 2,
        u32::MAX - 1,
    ];

    #[test]
    fn clone_id() {
        for n in INPUTS {
            let id = NodeId::from_usize(n as usize);
            // `With` copies the current value
            assert_eq!(id.clone_id(CloneInSemanticIds::With), id);
            // `Without` resets to the dummy ID `NodeId(0)`
            assert_eq!(id.clone_id(CloneInSemanticIds::Without), NodeId::from_usize(0));
        }
    }

    #[test]
    fn clone_cell_option_id() {
        // `None` is returned unchanged by both variants
        let cell = Cell::new(None);
        assert_eq!(NodeId::clone_cell_option_id(&cell, CloneInSemanticIds::With).get(), None);
        assert_eq!(NodeId::clone_cell_option_id(&cell, CloneInSemanticIds::Without).get(), None);

        for n in INPUTS {
            let id = NodeId::from_usize(n as usize);
            let cell = Cell::new(Some(id));
            // `With` copies the current value
            assert_eq!(
                NodeId::clone_cell_option_id(&cell, CloneInSemanticIds::With).get(),
                Some(id)
            );
            // `Without` resets to `None`
            assert_eq!(
                NodeId::clone_cell_option_id(&cell, CloneInSemanticIds::Without).get(),
                None
            );
        }
    }
