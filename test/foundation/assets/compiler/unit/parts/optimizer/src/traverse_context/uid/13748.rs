#[cfg(test)]
#[test]
fn uids() {
    // (&[ initial, ... ], &[ (name, expected_uid), ... ])
    #[expect(clippy::type_complexity)]
    let cases: &[(&[&str], &[(&str, &str)])] = &[
        (&[], &[("foo", "_foo"), ("foo", "_foo2"), ("foo", "_foo3")]),
        (
            &["foo", "foo0", "foo1", "foo2", "foo10", "_bar"],
            &[("foo", "_foo"), ("foo", "_foo2"), ("foo", "_foo3")],
        ),
        (
            &["_foo0", "_foo1", "__foo0", "____foo1", "_foo01", "_foo012345", "_foo000000"],
            &[("foo", "_foo"), ("foo", "_foo2"), ("foo", "_foo3")],
        ),
        (&[], &[("_foo", "_foo"), ("__foo", "_foo2"), ("_____foo", "_foo3")]),
        (&[], &[("_foo123", "_foo"), ("__foo456", "_foo2"), ("_____foo789", "_foo3")]),
        (&["_foo"], &[("foo", "_foo2"), ("foo", "_foo3"), ("foo", "_foo4")]),
        (&["_foo3"], &[("foo", "_foo4"), ("foo", "_foo5"), ("foo", "_foo6")]),
        (&["__foo"], &[("foo", "__foo2"), ("foo", "__foo3"), ("foo", "__foo4")]),
        (&["__foo8"], &[("foo", "__foo9"), ("foo", "__foo10"), ("foo", "__foo11")]),
        (&["_foo999", "____foo"], &[("foo", "____foo2"), ("foo", "____foo3"), ("foo", "____foo4")]),
        (
            &["_foo4294967293"],
            &[
                ("foo", "_foo4294967294"),
                ("foo", "_foo4294967295"),
                ("foo", "__foo2"),
                ("foo", "__foo3"),
            ],
        ),
        (
            &["___foo4294967293"],
            &[
                ("foo", "___foo4294967294"),
                ("foo", "___foo4294967295"),
                ("foo", "____foo2"),
                ("foo", "____foo3"),
            ],
        ),
        (&[], &[("_", "_"), ("_", "_2"), ("_", "_3")]),
        (
            &["_0", "_1", "__0", "____1", "_01", "_012345", "_000000"],
            &[("_", "_"), ("_", "_2"), ("_", "_3")],
        ),
        (&[], &[("___", "_"), ("_____", "_2"), ("_____", "_3")]),
        (&["_"], &[("_", "_2"), ("_", "_3"), ("_", "_4")]),
        (&["_4"], &[("_", "_5"), ("_", "_6"), ("_", "_7")]),
        (&["___"], &[("_", "___2"), ("_", "___3"), ("_", "___4")]),
        (&["___99"], &[("_", "___100"), ("_", "___101"), ("_", "___102")]),
        (&["_"], &[("_123", "_2"), ("__456", "_3"), ("___789", "_4")]),
    ];

    let allocator = Allocator::default();
    for &(used_names, created) in cases {
        let mut generator = UidGenerator { names: FxHashMap::default(), allocator: &allocator };
        for &used_name in used_names {
            generator.add(used_name);
        }

        for &(name, uid) in created {
            assert_eq!(generator.create(name), uid);
        }
    }
}
