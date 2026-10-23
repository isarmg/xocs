
    use super::{Colons, OffsetFormat, OffsetPrecision, Pad};
    use crate::FixedOffset;
    #[cfg(feature = "alloc")]
    use crate::{NaiveDate, NaiveTime, TimeZone, Timelike, Utc};

    #[cfg(feature = "alloc")]
    #[test]
    fn test_delayed_write_to() {
        let dt = crate::DateTime::from_timestamp(1643723400, 123456789).unwrap();
        let df = dt.format("%Y-%m-%d %H:%M:%S%.9f");

        let mut dt_str = String::new();

        df.write_to(&mut dt_str).unwrap();
        assert_eq!(dt_str, "2022-02-01 13:50:00.123456789");
    }

    #[cfg(all(feature = "std", feature = "unstable-locales", feature = "alloc"))]
    #[test]
    fn test_with_locale_delayed_write_to() {
        use crate::format::locales::Locale;
        use crate::DateTime;

        let dt = DateTime::from_timestamp(1643723400, 123456789).unwrap();
        let df = dt.format_localized("%A, %B %d, %Y", Locale::ja_JP);

        let mut dt_str = String::new();

        df.write_to(&mut dt_str).unwrap();

        assert_eq!(dt_str, "火曜日, 2月 01, 2022");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_date_format() {
        let d = NaiveDate::from_ymd_opt(2012, 3, 4).unwrap();
        assert_eq!(d.format("%Y,%C,%y,%G,%g").to_string(), "2012,20,12,2012,12");
        assert_eq!(d.format("%m,%b,%h,%B").to_string(), "03,Mar,Mar,March");
        assert_eq!(d.format("%q").to_string(), "1");
        assert_eq!(d.format("%d,%e").to_string(), "04, 4");
        assert_eq!(d.format("%U,%W,%V").to_string(), "10,09,09");
        assert_eq!(d.format("%a,%A,%w,%u").to_string(), "Sun,Sunday,0,7");
        assert_eq!(d.format("%j").to_string(), "064"); // since 2012 is a leap year
        assert_eq!(d.format("%D,%x").to_string(), "03/04/12,03/04/12");
        assert_eq!(d.format("%F").to_string(), "2012-03-04");
        assert_eq!(d.format("%v").to_string(), " 4-Mar-2012");
        assert_eq!(d.format("%t%n%%%n%t").to_string(), "\t\n%\n\t");

        // non-four-digit years
        assert_eq!(
            NaiveDate::from_ymd_opt(12345, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "+12345"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(1234, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "1234"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(123, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "0123"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(12, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "0012"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(1, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "0001"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(0, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "0000"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(-1, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "-0001"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(-12, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "-0012"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(-123, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "-0123"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(-1234, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "-1234"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(-12345, 1, 1)
                .unwrap()
                .format("%Y")
                .to_string(),
            "-12345"
        );

        // corner cases
        assert_eq!(
            NaiveDate::from_ymd_opt(2007, 12, 31)
                .unwrap()
                .format("%G,%g,%U,%W,%V")
                .to_string(),
            "2008,08,52,53,01"
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2010, 1, 3)
                .unwrap()
                .format("%G,%g,%U,%W,%V")
                .to_string(),
            "2009,09,01,00,53"
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_time_format() {
        let t = NaiveTime::from_hms_nano_opt(3, 5, 7, 98765432).unwrap();
        assert_eq!(
            t.format("%H,%k,%I,%l,%P,%p").to_string(),
            "03, 3,03, 3,am,AM"
        );
        assert_eq!(t.format("%M").to_string(), "05");
        assert_eq!(t.format("%S,%f,%.f").to_string(), "07,098765432,.098765432");
        assert_eq!(
            t.format("%.3f,%.6f,%.9f").to_string(),
            ".098,.098765,.098765432"
        );
        assert_eq!(t.format("%R").to_string(), "03:05");
        assert_eq!(t.format("%T,%X").to_string(), "03:05:07,03:05:07");
        assert_eq!(t.format("%r").to_string(), "03:05:07 AM");
        assert_eq!(t.format("%t%n%%%n%t").to_string(), "\t\n%\n\t");

        let t = NaiveTime::from_hms_micro_opt(3, 5, 7, 432100).unwrap();
        assert_eq!(t.format("%S,%f,%.f").to_string(), "07,432100000,.432100");
        assert_eq!(
            t.format("%.3f,%.6f,%.9f").to_string(),
            ".432,.432100,.432100000"
        );

        let t = NaiveTime::from_hms_milli_opt(3, 5, 7, 210).unwrap();
        assert_eq!(t.format("%S,%f,%.f").to_string(), "07,210000000,.210");
        assert_eq!(
            t.format("%.3f,%.6f,%.9f").to_string(),
            ".210,.210000,.210000000"
        );

        let t = NaiveTime::from_hms_opt(3, 5, 7).unwrap();
        assert_eq!(t.format("%S,%f,%.f").to_string(), "07,000000000,");
        assert_eq!(
            t.format("%.3f,%.6f,%.9f").to_string(),
            ".000,.000000,.000000000"
        );

        // corner cases
        assert_eq!(
            NaiveTime::from_hms_opt(13, 57, 9)
                .unwrap()
                .format("%r")
                .to_string(),
            "01:57:09 PM"
        );
        assert_eq!(
            NaiveTime::from_hms_milli_opt(23, 59, 59, 1_000)
                .unwrap()
                .format("%X")
                .to_string(),
            "23:59:60"
        );
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_datetime_format() {
        let dt = NaiveDate::from_ymd_opt(2010, 9, 8)
            .unwrap()
            .and_hms_milli_opt(7, 6, 54, 321)
            .unwrap();
        assert_eq!(dt.format("%c").to_string(), "Wed Sep  8 07:06:54 2010");
        assert_eq!(dt.format("%s").to_string(), "1283929614");
        assert_eq!(dt.format("%t%n%%%n%t").to_string(), "\t\n%\n\t");

        // a horror of leap second: coming near to you.
        let dt = NaiveDate::from_ymd_opt(2012, 6, 30)
            .unwrap()
            .and_hms_milli_opt(23, 59, 59, 1_000)
            .unwrap();
        assert_eq!(dt.format("%c").to_string(), "Sat Jun 30 23:59:60 2012");
        assert_eq!(dt.format("%s").to_string(), "1341100799"); // not 1341100800, it's intentional.
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_datetime_format_alignment() {
        let datetime = Utc
            .with_ymd_and_hms(2007, 1, 2, 12, 34, 56)
            .unwrap()
            .with_nanosecond(123456789)
            .unwrap();

        // Item::Literal, odd number of padding bytes.
        let percent = datetime.format("%%");
        assert_eq!("   %", format!("{percent:>4}"));
        assert_eq!("%   ", format!("{percent:<4}"));
        assert_eq!(" %  ", format!("{percent:^4}"));

        // Item::Numeric, custom non-ASCII padding character
        let year = datetime.format("%Y");
        assert_eq!("——2007", format!("{year:—>6}"));
        assert_eq!("2007——", format!("{year:—<6}"));
        assert_eq!("—2007—", format!("{year:—^6}"));

        // Item::Fixed
        let tz = datetime.format("%Z");
        assert_eq!("  UTC", format!("{tz:>5}"));
        assert_eq!("UTC  ", format!("{tz:<5}"));
        assert_eq!(" UTC ", format!("{tz:^5}"));

        // [Item::Numeric, Item::Space, Item::Literal, Item::Space, Item::Numeric]
        let ymd = datetime.format("%Y %B %d");
        assert_eq!("  2007 January 02", format!("{ymd:>17}"));
        assert_eq!("2007 January 02  ", format!("{ymd:<17}"));
        assert_eq!(" 2007 January 02 ", format!("{ymd:^17}"));

        // Truncated
        let time = datetime.format("%T%.6f");
        assert_eq!("12:34:56.1234", format!("{time:.13}"));
    }

    #[test]
    fn test_offset_formatting() {
        fn check_all(precision: OffsetPrecision, expected: [[&str; 7]; 12]) {
            fn check(
                precision: OffsetPrecision,
                colons: Colons,
                padding: Pad,
                allow_zulu: bool,
                offsets: [FixedOffset; 7],
                expected: [&str; 7],
            ) {
                let offset_format = OffsetFormat {
                    precision,
                    colons,
                    allow_zulu,
                    padding,
                };
                for (offset, expected) in offsets.iter().zip(expected.iter()) {
                    let mut output = String::new();
                    offset_format.format(&mut output, *offset).unwrap();
                    assert_eq!(&output, expected);
                }
            }
            // +03:45, -03:30, +11:00, -11:00:22, +02:34:26, -12:34:30, +00:00
            let offsets = [
                FixedOffset::east_opt(13_500).unwrap(),
                FixedOffset::east_opt(-12_600).unwrap(),
                FixedOffset::east_opt(39_600).unwrap(),
                FixedOffset::east_opt(-39_622).unwrap(),
                FixedOffset::east_opt(9266).unwrap(),
                FixedOffset::east_opt(-45270).unwrap(),
                FixedOffset::east_opt(0).unwrap(),
            ];
            check(
                precision,
                Colons::Colon,
                Pad::Zero,
                false,
                offsets,
                expected[0],
            );
            check(
                precision,
                Colons::Colon,
                Pad::Zero,
                true,
                offsets,
                expected[1],
            );
            check(
                precision,
                Colons::Colon,
                Pad::Space,
                false,
                offsets,
                expected[2],
            );
            check(
                precision,
                Colons::Colon,
                Pad::Space,
                true,
                offsets,
                expected[3],
            );
            check(
                precision,
                Colons::Colon,
                Pad::None,
                false,
                offsets,
                expected[4],
            );
            check(
                precision,
                Colons::Colon,
                Pad::None,
                true,
                offsets,
                expected[5],
            );
            check(
                precision,
                Colons::None,
                Pad::Zero,
                false,
                offsets,
                expected[6],
            );
            check(
                precision,
                Colons::None,
                Pad::Zero,
                true,
                offsets,
                expected[7],
            );
            check(
                precision,
                Colons::None,
                Pad::Space,
                false,
                offsets,
                expected[8],
            );
            check(
                precision,
                Colons::None,
                Pad::Space,
                true,
                offsets,
                expected[9],
            );
            check(
                precision,
                Colons::None,
                Pad::None,
                false,
                offsets,
                expected[10],
            );
            check(
                precision,
                Colons::None,
                Pad::None,
                true,
                offsets,
                expected[11],
            );
            // `Colons::Maybe` should format the same as `Colons::None`
            check(
                precision,
                Colons::Maybe,
                Pad::Zero,
                false,
                offsets,
                expected[6],
            );
            check(
                precision,
                Colons::Maybe,
                Pad::Zero,
                true,
                offsets,
                expected[7],
            );
            check(
                precision,
                Colons::Maybe,
                Pad::Space,
                false,
                offsets,
                expected[8],
            );
            check(
                precision,
                Colons::Maybe,
                Pad::Space,
                true,
                offsets,
                expected[9],
            );
            check(
                precision,
                Colons::Maybe,
                Pad::None,
                false,
                offsets,
                expected[10],
            );
            check(
                precision,
                Colons::Maybe,
                Pad::None,
                true,
                offsets,
                expected[11],
            );
        }
        check_all(
            OffsetPrecision::Hours,
            [
                ["+03", "-03", "+11", "-11", "+02", "-12", "+00"],
                ["+03", "-03", "+11", "-11", "+02", "-12", "Z"],
                [" +3", " -3", "+11", "-11", " +2", "-12", " +0"],
                [" +3", " -3", "+11", "-11", " +2", "-12", "Z"],
                ["+3", "-3", "+11", "-11", "+2", "-12", "+0"],
                ["+3", "-3", "+11", "-11", "+2", "-12", "Z"],
                ["+03", "-03", "+11", "-11", "+02", "-12", "+00"],
                ["+03", "-03", "+11", "-11", "+02", "-12", "Z"],
                [" +3", " -3", "+11", "-11", " +2", "-12", " +0"],
                [" +3", " -3", "+11", "-11", " +2", "-12", "Z"],
                ["+3", "-3", "+11", "-11", "+2", "-12", "+0"],
                ["+3", "-3", "+11", "-11", "+2", "-12", "Z"],
            ],
        );
        check_all(
            OffsetPrecision::Minutes,
            [
                [
                    "+03:45", "-03:30", "+11:00", "-11:00", "+02:34", "-12:35", "+00:00",
                ],
                [
                    "+03:45", "-03:30", "+11:00", "-11:00", "+02:34", "-12:35", "Z",
                ],
                [
                    " +3:45", " -3:30", "+11:00", "-11:00", " +2:34", "-12:35", " +0:00",
                ],
                [
                    " +3:45", " -3:30", "+11:00", "-11:00", " +2:34", "-12:35", "Z",
                ],
                [
                    "+3:45", "-3:30", "+11:00", "-11:00", "+2:34", "-12:35", "+0:00",
                ],
                ["+3:45", "-3:30", "+11:00", "-11:00", "+2:34", "-12:35", "Z"],
                [
                    "+0345", "-0330", "+1100", "-1100", "+0234", "-1235", "+0000",
                ],
                ["+0345", "-0330", "+1100", "-1100", "+0234", "-1235", "Z"],
                [
                    " +345", " -330", "+1100", "-1100", " +234", "-1235", " +000",
                ],
                [" +345", " -330", "+1100", "-1100", " +234", "-1235", "Z"],
                ["+345", "-330", "+1100", "-1100", "+234", "-1235", "+000"],
                ["+345", "-330", "+1100", "-1100", "+234", "-1235", "Z"],
            ],
        );
        #[rustfmt::skip]
        check_all(
            OffsetPrecision::Seconds,
            [
                ["+03:45:00", "-03:30:00", "+11:00:00", "-11:00:22", "+02:34:26", "-12:34:30", "+00:00:00"],
                ["+03:45:00", "-03:30:00", "+11:00:00", "-11:00:22", "+02:34:26", "-12:34:30", "Z"],
                [" +3:45:00", " -3:30:00", "+11:00:00", "-11:00:22", " +2:34:26", "-12:34:30", " +0:00:00"],
                [" +3:45:00", " -3:30:00", "+11:00:00", "-11:00:22", " +2:34:26", "-12:34:30", "Z"],
                ["+3:45:00", "-3:30:00", "+11:00:00", "-11:00:22", "+2:34:26", "-12:34:30", "+0:00:00"],
                ["+3:45:00", "-3:30:00", "+11:00:00", "-11:00:22", "+2:34:26", "-12:34:30", "Z"],
                ["+034500", "-033000", "+110000", "-110022", "+023426", "-123430", "+000000"],
                ["+034500", "-033000", "+110000", "-110022", "+023426", "-123430", "Z"],
                [" +34500", " -33000", "+110000", "-110022", " +23426", "-123430", " +00000"],
                [" +34500", " -33000", "+110000", "-110022", " +23426", "-123430", "Z"],
                ["+34500", "-33000", "+110000", "-110022", "+23426", "-123430", "+00000"],
                ["+34500", "-33000", "+110000", "-110022", "+23426", "-123430", "Z"],
            ],
        );
        check_all(
            OffsetPrecision::OptionalMinutes,
            [
                ["+03:45", "-03:30", "+11", "-11", "+02:34", "-12:35", "+00"],
                ["+03:45", "-03:30", "+11", "-11", "+02:34", "-12:35", "Z"],
                [" +3:45", " -3:30", "+11", "-11", " +2:34", "-12:35", " +0"],
                [" +3:45", " -3:30", "+11", "-11", " +2:34", "-12:35", "Z"],
                ["+3:45", "-3:30", "+11", "-11", "+2:34", "-12:35", "+0"],
                ["+3:45", "-3:30", "+11", "-11", "+2:34", "-12:35", "Z"],
                ["+0345", "-0330", "+11", "-11", "+0234", "-1235", "+00"],
                ["+0345", "-0330", "+11", "-11", "+0234", "-1235", "Z"],
                [" +345", " -330", "+11", "-11", " +234", "-1235", " +0"],
                [" +345", " -330", "+11", "-11", " +234", "-1235", "Z"],
                ["+345", "-330", "+11", "-11", "+234", "-1235", "+0"],
                ["+345", "-330", "+11", "-11", "+234", "-1235", "Z"],
            ],
        );
        check_all(
            OffsetPrecision::OptionalSeconds,
            [
                [
                    "+03:45",
                    "-03:30",
                    "+11:00",
                    "-11:00:22",
                    "+02:34:26",
                    "-12:34:30",
                    "+00:00",
                ],
                [
                    "+03:45",
                    "-03:30",
                    "+11:00",
                    "-11:00:22",
                    "+02:34:26",
                    "-12:34:30",
                    "Z",
                ],
                [
                    " +3:45",
                    " -3:30",
                    "+11:00",
                    "-11:00:22",
                    " +2:34:26",
                    "-12:34:30",
                    " +0:00",
                ],
                [
                    " +3:45",
                    " -3:30",
                    "+11:00",
                    "-11:00:22",
                    " +2:34:26",
                    "-12:34:30",
                    "Z",
                ],
                [
                    "+3:45",
                    "-3:30",
                    "+11:00",
                    "-11:00:22",
                    "+2:34:26",
                    "-12:34:30",
                    "+0:00",
                ],
                [
                    "+3:45",
                    "-3:30",
                    "+11:00",
                    "-11:00:22",
                    "+2:34:26",
                    "-12:34:30",
                    "Z",
                ],
                [
                    "+0345", "-0330", "+1100", "-110022", "+023426", "-123430", "+0000",
                ],
                [
                    "+0345", "-0330", "+1100", "-110022", "+023426", "-123430", "Z",
                ],
                [
                    " +345", " -330", "+1100", "-110022", " +23426", "-123430", " +000",
                ],
                [
                    " +345", " -330", "+1100", "-110022", " +23426", "-123430", "Z",
                ],
                [
                    "+345", "-330", "+1100", "-110022", "+23426", "-123430", "+000",
                ],
                ["+345", "-330", "+1100", "-110022", "+23426", "-123430", "Z"],
            ],
        );
        check_all(
            OffsetPrecision::OptionalMinutesAndSeconds,
            [
                [
                    "+03:45",
                    "-03:30",
                    "+11",
                    "-11:00:22",
                    "+02:34:26",
                    "-12:34:30",
                    "+00",
                ],
                [
                    "+03:45",
                    "-03:30",
                    "+11",
                    "-11:00:22",
                    "+02:34:26",
                    "-12:34:30",
                    "Z",
                ],
                [
                    " +3:45",
                    " -3:30",
                    "+11",
                    "-11:00:22",
                    " +2:34:26",
                    "-12:34:30",
                    " +0",
                ],
                [
                    " +3:45",
                    " -3:30",
                    "+11",
                    "-11:00:22",
                    " +2:34:26",
                    "-12:34:30",
                    "Z",
                ],
                [
                    "+3:45",
                    "-3:30",
                    "+11",
                    "-11:00:22",
                    "+2:34:26",
                    "-12:34:30",
                    "+0",
                ],
                [
                    "+3:45",
                    "-3:30",
                    "+11",
                    "-11:00:22",
                    "+2:34:26",
                    "-12:34:30",
                    "Z",
                ],
                [
                    "+0345", "-0330", "+11", "-110022", "+023426", "-123430", "+00",
                ],
                [
                    "+0345", "-0330", "+11", "-110022", "+023426", "-123430", "Z",
                ],
                [
                    " +345", " -330", "+11", "-110022", " +23426", "-123430", " +0",
                ],
                [
                    " +345", " -330", "+11", "-110022", " +23426", "-123430", "Z",
                ],
                ["+345", "-330", "+11", "-110022", "+23426", "-123430", "+0"],
                ["+345", "-330", "+11", "-110022", "+23426", "-123430", "Z"],
            ],
        );
    }
