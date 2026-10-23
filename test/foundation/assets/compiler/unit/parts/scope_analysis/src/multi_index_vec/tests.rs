
    crate::typed_index::define_index_type! {
        struct TestId = usize;
        MAX_INDEX = usize::MAX;
    }

    multi_index_vec! {
        struct TestTable<TestId> {
            values => values_mut: u32,
        }
    }

    crate::typed_index::define_index_type! {
        struct SmallId = u8;
        MAX_INDEX = 3;
    }

    multi_index_vec! {
        struct SmallTable<SmallId> {
            values => values_mut: u32,
        }
    }

    crate::typed_index::define_index_type! {
        struct StringId = usize;
        MAX_INDEX = usize::MAX;
    }

    multi_index_vec! {
        struct StringTable<StringId> {
            values => values_mut: String,
        }
    }

    #[test]
    fn iter_ids_are_valid() {
        let mut table = TestTable::new();
        assert!(table.is_empty());
        table.reserve(3);
        table.push(1);
        table.push(2);
        table.push(3);
        assert_eq!(table.len(), 3);

        let ids = table.iter_ids().collect::<Vec<_>>();
        assert_eq!(ids.len(), 3);
        assert_eq!(usize::from(ids[0]), 0);
        assert_eq!(usize::from(ids[1]), 1);
        assert_eq!(usize::from(ids[2]), 2);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn accessor_panics_on_invalid_index() {
        let mut table = TestTable::new();
        table.push(1);
        let _ = table.values(TestId::from(1usize));
    }

    #[test]
    #[should_panic(expected = "capacity exceeds index type range")]
    fn reserve_panics_when_exceeding_index_range() {
        let mut table = SmallTable::new();
        assert!(table.is_empty());
        assert_eq!(table.len(), 0);
        let _ = table.iter_ids().count();
        table.reserve(5);
    }

    #[test]
    #[should_panic(expected = "capacity exceeds index type range")]
    fn push_panics_when_exceeding_index_range() {
        let mut table = SmallTable::new();
        let id = table.push(1);
        assert_eq!(*table.values(id), 1);
        table.push(2);
        table.push(3);
        table.push(4);
        table.push(5);
    }

    #[test]
    fn clone_clones_non_copy_fields() {
        let mut table = StringTable::new();
        assert!(table.is_empty());
        table.reserve(1);
        let id = table.push(String::from("a"));
        assert_eq!(table.len(), 1);
        let _ = table.iter_ids().count();
        let mut clone = table.clone();

        table.values_mut(id).push('1');
        clone.values_mut(id).push('2');

        assert_eq!(table.values(id), "a1");
        assert_eq!(clone.values(id), "a2");
    }
