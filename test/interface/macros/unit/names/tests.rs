use super::*;
#[test]
fn names_preserve_boundaries() {
    for (value, lower, title) in [
        ("HTTPRequest", "http_request", "HttpRequest"),
        ("ShowLet", "show_let", "ShowLet"),
        ("item_2d", "item_2_d", "Item2D"),
        ("Été_value", "été_value", "ÉtéValue"),
        ("foo__bar", "foo__bar", "FooBar"),
        ("", "", ""),
    ] {
        assert_eq!(snake(value), lower);
        assert_eq!(pascal(value), title);
    }
}
