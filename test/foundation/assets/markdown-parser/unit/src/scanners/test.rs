
    use super::*;
    #[test]
    fn overflow_list() {
        assert!(
            scan_listitem(b"4444444444444444444444444444444444444444444444444444444444!").is_none()
        );
    }

    #[test]
    fn overflow_by_addition() {
        assert!(scan_listitem(b"1844674407370955161615!").is_none());
    }

    #[test]
    fn good_emails() {
        const EMAILS: &[&str] = &[
            "<a@b.c>",
            "<a@b>",
            "<a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-@example.com>",
            "<a@sixty-three-letters-in-this-identifier-----------------------63>",
        ];
        for email in EMAILS {
            assert!(scan_email(email, 1).is_some());
        }
    }

    #[test]
    fn bad_emails() {
        const EMAILS: &[&str] = &[
            "<@b.c>",
            "<foo@-example.com>",
            "<foo@example-.com>",
            "<a@notrailingperiod.>",
            "<a(noparens)@example.com>",
            "<\"noquotes\"@example.com>",
            "<a@sixty-four-letters-in-this-identifier-------------------------64>",
        ];
        for email in EMAILS {
            assert!(scan_email(email, 1).is_none());
        }
    }
