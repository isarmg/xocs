
    #[cfg(feature = "clock")]
    use crate::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};

    #[test]
    #[allow(deprecated)]
    #[cfg(feature = "clock")]
    fn test_type_sizes() {
        use core::mem::size_of;
        assert_eq!(size_of::<NaiveDate>(), 4);
        assert_eq!(size_of::<Option<NaiveDate>>(), 4);
        assert_eq!(size_of::<NaiveTime>(), 8);
        assert_eq!(size_of::<Option<NaiveTime>>(), 12);
        assert_eq!(size_of::<NaiveDateTime>(), 12);
        assert_eq!(size_of::<Option<NaiveDateTime>>(), 12);

        assert_eq!(size_of::<DateTime<Utc>>(), 12);
        assert_eq!(size_of::<DateTime<FixedOffset>>(), 16);
        assert_eq!(size_of::<DateTime<Local>>(), 16);
        assert_eq!(size_of::<Option<DateTime<FixedOffset>>>(), 16);
    }
