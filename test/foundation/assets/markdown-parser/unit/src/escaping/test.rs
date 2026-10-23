
    pub use super::{escape_href, escape_html, escape_html_body_text};

    #[test]
    fn check_href_escape() {
        let mut s = String::new();
        escape_href(&mut s, "&^_").unwrap();
        assert_eq!(s.as_str(), "&amp;^_");
    }

    #[test]
    fn check_attr_escape() {
        let mut s = String::new();
        escape_html(&mut s, r##"&^"'_"##).unwrap();
        assert_eq!(s.as_str(), "&amp;^&quot;&#39;_");
    }

    #[test]
    fn check_body_escape() {
        let mut s = String::new();
        escape_html_body_text(&mut s, r##"&^"'_"##).unwrap();
        assert_eq!(s.as_str(), r##"&amp;^"'_"##);
    }
