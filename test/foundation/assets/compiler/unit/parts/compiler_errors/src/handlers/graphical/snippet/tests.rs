
    use super::GraphicalReportHandler;

    #[test]
    fn direct_locations_match_standard_formatting() {
        for (source_name, line, column) in
            [(None, 1, 1), (Some("src/火.ts"), usize::MAX, usize::MAX)]
        {
            let expected = source_name.map_or_else(
                || format!("[{line}:{column}]\n"),
                |source_name| format!("[{source_name}:{line}:{column}]\n"),
            );
            let mut actual = String::new();
            GraphicalReportHandler::write_location(&mut actual, source_name, line, column).unwrap();
            assert_eq!(actual, expected);
        }
    }
