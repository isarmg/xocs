
    use super::*;
    use crate::compiler_errors::GraphicalTheme;

    #[test]
    fn direct_plain_line_numbers_match_standard_formatting() {
        let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none());
        for (width, linum) in [(3, 1), (1, usize::MAX)] {
            let mut actual = String::new();
            handler.write_linum(&mut actual, width, linum).unwrap();
            assert_eq!(actual, format!(" {linum:width$} | "));
        }
    }
