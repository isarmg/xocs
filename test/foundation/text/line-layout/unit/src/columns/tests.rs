
    use super::*;
    use alloc::vec;

    #[test]
    fn wrap_columns_empty_text() {
        assert_eq!(wrap_columns("", 1, 10, "| ", "", " |"), vec!["|        |"]);
    }

    #[test]
    fn wrap_columns_single_column() {
        assert_eq!(
            wrap_columns("Foo", 3, 30, "| ", " | ", " |"),
            vec!["| Foo    |        |          |"]
        );
    }

    #[test]
    fn wrap_columns_uneven_columns() {
        // The gaps take up a total of 5 columns, so the columns are
        // (21 - 5)/4 = 4 columns wide:
        assert_eq!(
            wrap_columns("Foo Bar Baz Quux", 4, 21, "|", "|", "|"),
            vec!["|Foo |Bar |Baz |Quux|"]
        );
        // As the total width increases, the last column absorbs the
        // excess width:
        assert_eq!(
            wrap_columns("Foo Bar Baz Quux", 4, 24, "|", "|", "|"),
            vec!["|Foo |Bar |Baz |Quux   |"]
        );
        // Finally, when the width is 25, the columns can be resized
        // to a width of (25 - 5)/4 = 5 columns:
        assert_eq!(
            wrap_columns("Foo Bar Baz Quux", 4, 25, "|", "|", "|"),
            vec!["|Foo  |Bar  |Baz  |Quux |"]
        );
    }

    #[test]
    #[cfg(all(feature = "unicode-width", feature = "smawk"))]
    fn wrap_columns_with_emojis() {
        assert_eq!(
            wrap_columns(
                "Words and a few emojis 😍 wrapped in ⓶ columns",
                2,
                30,
                "✨ ",
                " ⚽ ",
                " 👀"
            ),
            vec![
                "✨ Words      ⚽ wrapped in 👀",
                "✨ and a few  ⚽ ⓶ columns  👀",
                "✨ emojis 😍  ⚽            👀"
            ]
        );
    }

    #[test]
    #[cfg(all(feature = "unicode-width", not(feature = "smawk")))]
    fn wrap_columns_with_emojis_without_smawk() {
        assert_eq!(
            wrap_columns(
                "Words and a few emojis 😍 wrapped in ⓶ columns",
                2,
                30,
                "✨ ",
                " ⚽ ",
                " 👀"
            ),
            vec![
                "✨ Words and  ⚽ wrapped in 👀",
                "✨ a few      ⚽ ⓶ columns  👀",
                "✨ emojis 😍  ⚽            👀"
            ]
        );
    }

    #[test]
    fn wrap_columns_big_gaps() {
        // The column width shrinks to 1 because the gaps take up all
        // the space.
        assert_eq!(
            wrap_columns("xyz", 2, 10, "----> ", " !!! ", " <----"),
            vec![
                "----> x !!! z <----", //
                "----> y !!!   <----"
            ]
        );
    }

    #[test]
    #[should_panic]
    fn wrap_columns_panic_with_zero_columns() {
        wrap_columns("", 0, 10, "", "", "");
    }

    #[test]
    fn inner_width_with_normal_gaps() {
        // The gaps take up a total of 5 columns.
        assert_eq!(inner_width(21, 4, "|", "|", "|"), 16);
    }

    #[test]
    fn inner_width_with_huge_number_of_columns() {
        // The combined width of the middle gaps overflows a `usize`,
        // which leaves no room at all for the columns. The number of
        // columns is chosen so that a wrapping multiplication would
        // wrap all the way around to zero.
        assert_eq!(inner_width(80, usize::MAX / 2 + 2, "", "xx", ""), 0);
    }
