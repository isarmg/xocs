
    use super::StrftimeItems;
    use crate::format::Item::{self, Literal, Space};
    #[cfg(feature = "unstable-locales")]
    use crate::format::Locale;
    use crate::format::{fixed, internal_fixed, num, num0, nums};
    use crate::format::{Fixed, InternalInternal, Numeric::*};
    #[cfg(feature = "alloc")]
    use crate::{DateTime, FixedOffset, NaiveDate, TimeZone, Timelike, Utc};

    #[test]
    fn test_strftime_items() {
        fn parse_and_collect(s: &str) -> Vec<Item<'_>> {
            // map any error into `[Item::Error]`. useful for easy testing.
            eprintln!("test_strftime_items: parse_and_collect({s:?})");
            let items = StrftimeItems::new(s);
            let items = items.map(|spec| {
                if spec == Item::Error {
                    None
                } else {
                    Some(spec)
                }
            });
            items
                .collect::<Option<Vec<_>>>()
                .unwrap_or_else(|| vec![Item::Error])
        }

        assert_eq!(parse_and_collect(""), []);
        assert_eq!(parse_and_collect(" "), [Space(" ")]);
        assert_eq!(parse_and_collect("  "), [Space("  ")]);
        // ne!
        assert_ne!(parse_and_collect("  "), [Space(" "), Space(" ")]);
        // eq!
        assert_eq!(parse_and_collect("  "), [Space("  ")]);
        assert_eq!(parse_and_collect("a"), [Literal("a")]);
        assert_eq!(parse_and_collect("ab"), [Literal("ab")]);
        assert_eq!(parse_and_collect("😽"), [Literal("😽")]);
        assert_eq!(parse_and_collect("a😽"), [Literal("a😽")]);
        assert_eq!(parse_and_collect("😽a"), [Literal("😽a")]);
        assert_eq!(parse_and_collect(" 😽"), [Space(" "), Literal("😽")]);
        assert_eq!(parse_and_collect("😽 "), [Literal("😽"), Space(" ")]);
        // ne!
        assert_ne!(parse_and_collect("😽😽"), [Literal("😽")]);
        assert_ne!(parse_and_collect("😽"), [Literal("😽😽")]);
        assert_ne!(parse_and_collect("😽😽"), [Literal("😽😽"), Literal("😽")]);
        // eq!
        assert_eq!(parse_and_collect("😽😽"), [Literal("😽😽")]);
        assert_eq!(parse_and_collect(" \t\n\r "), [Space(" \t\n\r ")]);
        assert_eq!(parse_and_collect("hello?"), [Literal("hello?")]);
        assert_eq!(
            parse_and_collect("a  b\t\nc"),
            [
                Literal("a"),
                Space("  "),
                Literal("b"),
                Space("\t\n"),
                Literal("c")
            ]
        );
        assert_eq!(parse_and_collect("100%%"), [Literal("100"), Literal("%")]);
        assert_eq!(
            parse_and_collect("100%% ok"),
            [Literal("100"), Literal("%"), Space(" "), Literal("ok")]
        );
        assert_eq!(
            parse_and_collect("%%PDF-1.0"),
            [Literal("%"), Literal("PDF-1.0")]
        );
        assert_eq!(
            parse_and_collect("%Y-%m-%d"),
            [
                num0(Year),
                Literal("-"),
                num0(Month),
                Literal("-"),
                num0(Day)
            ]
        );
        assert_eq!(parse_and_collect("😽   "), [Literal("😽"), Space("   ")]);
        assert_eq!(parse_and_collect("😽😽"), [Literal("😽😽")]);
        assert_eq!(parse_and_collect("😽😽😽"), [Literal("😽😽😽")]);
        assert_eq!(
            parse_and_collect("😽😽 😽"),
            [Literal("😽😽"), Space(" "), Literal("😽")]
        );
        assert_eq!(
            parse_and_collect("😽😽a 😽"),
            [Literal("😽😽a"), Space(" "), Literal("😽")]
        );
        assert_eq!(
            parse_and_collect("😽😽a b😽"),
            [Literal("😽😽a"), Space(" "), Literal("b😽")]
        );
        assert_eq!(
            parse_and_collect("😽😽a b😽c"),
            [Literal("😽😽a"), Space(" "), Literal("b😽c")]
        );
        assert_eq!(
            parse_and_collect("😽😽   "),
            [Literal("😽😽"), Space("   ")]
        );
        assert_eq!(
            parse_and_collect("😽😽   😽"),
            [Literal("😽😽"), Space("   "), Literal("😽")]
        );
        assert_eq!(parse_and_collect("   😽"), [Space("   "), Literal("😽")]);
        assert_eq!(
            parse_and_collect("   😽 "),
            [Space("   "), Literal("😽"), Space(" ")]
        );
        assert_eq!(
            parse_and_collect("   😽 😽"),
            [Space("   "), Literal("😽"), Space(" "), Literal("😽")]
        );
        assert_eq!(
            parse_and_collect("   😽 😽 "),
            [
                Space("   "),
                Literal("😽"),
                Space(" "),
                Literal("😽"),
                Space(" ")
            ]
        );
        assert_eq!(
            parse_and_collect("   😽  😽 "),
            [
                Space("   "),
                Literal("😽"),
                Space("  "),
                Literal("😽"),
                Space(" ")
            ]
        );
        assert_eq!(
            parse_and_collect("   😽  😽😽 "),
            [
                Space("   "),
                Literal("😽"),
                Space("  "),
                Literal("😽😽"),
                Space(" ")
            ]
        );
        assert_eq!(
            parse_and_collect("   😽😽"),
            [Space("   "), Literal("😽😽")]
        );
        assert_eq!(
            parse_and_collect("   😽😽 "),
            [Space("   "), Literal("😽😽"), Space(" ")]
        );
        assert_eq!(
            parse_and_collect("   😽😽    "),
            [Space("   "), Literal("😽😽"), Space("    ")]
        );
        assert_eq!(
            parse_and_collect("   😽😽    "),
            [Space("   "), Literal("😽😽"), Space("    ")]
        );
        assert_eq!(
            parse_and_collect(" 😽😽    "),
            [Space(" "), Literal("😽😽"), Space("    ")]
        );
        assert_eq!(
            parse_and_collect(" 😽 😽😽    "),
            [
                Space(" "),
                Literal("😽"),
                Space(" "),
                Literal("😽😽"),
                Space("    ")
            ]
        );
        assert_eq!(
            parse_and_collect(" 😽 😽はい😽    ハンバーガー"),
            [
                Space(" "),
                Literal("😽"),
                Space(" "),
                Literal("😽はい😽"),
                Space("    "),
                Literal("ハンバーガー")
            ]
        );
        assert_eq!(
            parse_and_collect("%%😽%%😽"),
            [Literal("%"), Literal("😽"), Literal("%"), Literal("😽")]
        );
        assert_eq!(
            parse_and_collect("%Y--%m"),
            [num0(Year), Literal("--"), num0(Month)]
        );
        assert_eq!(parse_and_collect("[%F]"), parse_and_collect("[%Y-%m-%d]"));
        assert_eq!(
            parse_and_collect("100%%😽"),
            [Literal("100"), Literal("%"), Literal("😽")]
        );
        assert_eq!(
            parse_and_collect("100%%😽%%a"),
            [
                Literal("100"),
                Literal("%"),
                Literal("😽"),
                Literal("%"),
                Literal("a")
            ]
        );
        assert_eq!(
            parse_and_collect("😽100%%"),
            [Literal("😽100"), Literal("%")]
        );
        assert_eq!(
            parse_and_collect("%m %d"),
            [num0(Month), Space(" "), num0(Day)]
        );
        assert_eq!(parse_and_collect("%"), [Item::Error]);
        assert_eq!(parse_and_collect("%%"), [Literal("%")]);
        assert_eq!(parse_and_collect("%%%"), [Item::Error]);
        assert_eq!(parse_and_collect("%a"), [fixed(Fixed::ShortWeekdayName)]);
        assert_eq!(
            parse_and_collect("%aa"),
            [fixed(Fixed::ShortWeekdayName), Literal("a")]
        );
        assert_eq!(parse_and_collect("%%a%"), [Item::Error]);
        assert_eq!(parse_and_collect("%😽"), [Item::Error]);
        assert_eq!(parse_and_collect("%😽😽"), [Item::Error]);
        assert_eq!(parse_and_collect("%%%%"), [Literal("%"), Literal("%")]);
        assert_eq!(
            parse_and_collect("%%%%ハンバーガー"),
            [Literal("%"), Literal("%"), Literal("ハンバーガー")]
        );
        assert_eq!(parse_and_collect("foo%?"), [Item::Error]);
        assert_eq!(parse_and_collect("bar%42"), [Item::Error]);
        assert_eq!(parse_and_collect("quux% +"), [Item::Error]);
        assert_eq!(parse_and_collect("%.Z"), [Item::Error]);
        assert_eq!(parse_and_collect("%:Z"), [Item::Error]);
        assert_eq!(parse_and_collect("%-Z"), [Item::Error]);
        assert_eq!(parse_and_collect("%0Z"), [Item::Error]);
        assert_eq!(parse_and_collect("%_Z"), [Item::Error]);
        assert_eq!(parse_and_collect("%.j"), [Item::Error]);
        assert_eq!(parse_and_collect("%:j"), [Item::Error]);
        assert_eq!(parse_and_collect("%-j"), [num(Ordinal)]);
        assert_eq!(parse_and_collect("%0j"), [num0(Ordinal)]);
        assert_eq!(parse_and_collect("%_j"), [nums(Ordinal)]);
        assert_eq!(parse_and_collect("%.e"), [Item::Error]);
        assert_eq!(parse_and_collect("%:e"), [Item::Error]);
        assert_eq!(parse_and_collect("%-e"), [num(Day)]);
        assert_eq!(parse_and_collect("%0e"), [num0(Day)]);
        assert_eq!(parse_and_collect("%_e"), [nums(Day)]);
        assert_eq!(parse_and_collect("%z"), [fixed(Fixed::TimezoneOffset)]);
        assert_eq!(
            parse_and_collect("%:z"),
            [fixed(Fixed::TimezoneOffsetColon)]
        );
        assert_eq!(parse_and_collect("%Z"), [fixed(Fixed::TimezoneName)]);
        assert_eq!(
            parse_and_collect("%ZZZZ"),
            [fixed(Fixed::TimezoneName), Literal("ZZZ")]
        );
        assert_eq!(
            parse_and_collect("%Z😽"),
            [fixed(Fixed::TimezoneName), Literal("😽")]
        );
        assert_eq!(
            parse_and_collect("%#z"),
            [internal_fixed(InternalInternal::TimezoneOffsetPermissive)]
        );
        assert_eq!(parse_and_collect("%#m"), [Item::Error]);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn test_strftime_docs() {
        let dt = FixedOffset::east_opt(34200)
            .unwrap()
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2001, 7, 8)
                    .unwrap()
                    .and_hms_nano_opt(0, 34, 59, 1_026_490_708)
                    .unwrap(),
            )
            .unwrap();

        // date specifiers
        assert_eq!(dt.format("%Y").to_string(), "2001");
        assert_eq!(dt.format("%C").to_string(), "20");
        assert_eq!(dt.format("%y").to_string(), "01");
        assert_eq!(dt.format("%q").to_string(), "3");
        assert_eq!(dt.format("%m").to_string(), "07");
        assert_eq!(dt.format("%b").to_string(), "Jul");
        assert_eq!(dt.format("%B").to_string(), "July");
        assert_eq!(dt.format("%h").to_string(), "Jul");
        assert_eq!(dt.format("%d").to_string(), "08");
        assert_eq!(dt.format("%e").to_string(), " 8");
        assert_eq!(dt.format("%e").to_string(), dt.format("%_d").to_string());
        assert_eq!(dt.format("%a").to_string(), "Sun");
        assert_eq!(dt.format("%A").to_string(), "Sunday");
        assert_eq!(dt.format("%w").to_string(), "0");
        assert_eq!(dt.format("%u").to_string(), "7");
        assert_eq!(dt.format("%U").to_string(), "27");
        assert_eq!(dt.format("%W").to_string(), "27");
        assert_eq!(dt.format("%G").to_string(), "2001");
        assert_eq!(dt.format("%g").to_string(), "01");
        assert_eq!(dt.format("%V").to_string(), "27");
        assert_eq!(dt.format("%j").to_string(), "189");
        assert_eq!(dt.format("%D").to_string(), "07/08/01");
        assert_eq!(dt.format("%x").to_string(), "07/08/01");
        assert_eq!(dt.format("%F").to_string(), "2001-07-08");
        assert_eq!(dt.format("%v").to_string(), " 8-Jul-2001");

        // time specifiers
        assert_eq!(dt.format("%H").to_string(), "00");
        assert_eq!(dt.format("%k").to_string(), " 0");
        assert_eq!(dt.format("%k").to_string(), dt.format("%_H").to_string());
        assert_eq!(dt.format("%I").to_string(), "12");
        assert_eq!(dt.format("%l").to_string(), "12");
        assert_eq!(dt.format("%l").to_string(), dt.format("%_I").to_string());
        assert_eq!(dt.format("%P").to_string(), "am");
        assert_eq!(dt.format("%p").to_string(), "AM");
        assert_eq!(dt.format("%M").to_string(), "34");
        assert_eq!(dt.format("%S").to_string(), "60");
        assert_eq!(dt.format("%f").to_string(), "026490708");
        assert_eq!(dt.format("%.f").to_string(), ".026490708");
        assert_eq!(
            dt.with_nanosecond(1_026_490_000)
                .unwrap()
                .format("%.f")
                .to_string(),
            ".026490"
        );
        assert_eq!(dt.format("%.3f").to_string(), ".026");
        assert_eq!(dt.format("%.6f").to_string(), ".026490");
        assert_eq!(dt.format("%.9f").to_string(), ".026490708");
        assert_eq!(dt.format("%3f").to_string(), "026");
        assert_eq!(dt.format("%6f").to_string(), "026490");
        assert_eq!(dt.format("%9f").to_string(), "026490708");
        assert_eq!(dt.format("%R").to_string(), "00:34");
        assert_eq!(dt.format("%T").to_string(), "00:34:60");
        assert_eq!(dt.format("%X").to_string(), "00:34:60");
        assert_eq!(dt.format("%r").to_string(), "12:34:60 AM");

        // time zone specifiers
        //assert_eq!(dt.format("%Z").to_string(), "ACST");
        assert_eq!(dt.format("%z").to_string(), "+0930");
        assert_eq!(dt.format("%:z").to_string(), "+09:30");
        assert_eq!(dt.format("%::z").to_string(), "+09:30:00");
        assert_eq!(dt.format("%:::z").to_string(), "+09");

        // date & time specifiers
        assert_eq!(dt.format("%c").to_string(), "Sun Jul  8 00:34:60 2001");
        assert_eq!(
            dt.format("%+").to_string(),
            "2001-07-08T00:34:60.026490708+09:30"
        );

        assert_eq!(
            dt.with_timezone(&Utc).format("%+").to_string(),
            "2001-07-07T15:04:60.026490708+00:00"
        );
        assert_eq!(
            dt.with_timezone(&Utc),
            DateTime::parse_from_str("2001-07-07T15:04:60.026490708Z", "%+").unwrap()
        );
        assert_eq!(
            dt.with_timezone(&Utc),
            DateTime::parse_from_str("2001-07-07T15:04:60.026490708UTC", "%+").unwrap()
        );
        assert_eq!(
            dt.with_timezone(&Utc),
            DateTime::parse_from_str("2001-07-07t15:04:60.026490708utc", "%+").unwrap()
        );

        assert_eq!(
            dt.with_nanosecond(1_026_490_000)
                .unwrap()
                .format("%+")
                .to_string(),
            "2001-07-08T00:34:60.026490+09:30"
        );
        assert_eq!(dt.format("%s").to_string(), "994518299");

        // special specifiers
        assert_eq!(dt.format("%t").to_string(), "\t");
        assert_eq!(dt.format("%n").to_string(), "\n");
        assert_eq!(dt.format("%%").to_string(), "%");

        // complex format specifiers
        assert_eq!(
            dt.format("  %Y%d%m%%%%%t%H%M%S\t").to_string(),
            "  20010807%%\t003460\t"
        );
        assert_eq!(
            dt.format("  %Y%d%m%%%%%t%H:%P:%M%S%:::z\t").to_string(),
            "  20010807%%\t00:am:3460+09\t"
        );
    }

    #[test]
    #[cfg(all(feature = "unstable-locales", feature = "alloc"))]
    fn test_strftime_docs_localized() {
        let dt = FixedOffset::east_opt(34200)
            .unwrap()
            .with_ymd_and_hms(2001, 7, 8, 0, 34, 59)
            .unwrap()
            .with_nanosecond(1_026_490_708)
            .unwrap();

        // date specifiers
        assert_eq!(dt.format_localized("%b", Locale::fr_BE).to_string(), "jui");
        assert_eq!(
            dt.format_localized("%B", Locale::fr_BE).to_string(),
            "juillet"
        );
        assert_eq!(dt.format_localized("%h", Locale::fr_BE).to_string(), "jui");
        assert_eq!(dt.format_localized("%a", Locale::fr_BE).to_string(), "dim");
        assert_eq!(
            dt.format_localized("%A", Locale::fr_BE).to_string(),
            "dimanche"
        );
        assert_eq!(
            dt.format_localized("%D", Locale::fr_BE).to_string(),
            "07/08/01"
        );
        assert_eq!(
            dt.format_localized("%x", Locale::fr_BE).to_string(),
            "08/07/01"
        );
        assert_eq!(
            dt.format_localized("%F", Locale::fr_BE).to_string(),
            "2001-07-08"
        );
        assert_eq!(
            dt.format_localized("%v", Locale::fr_BE).to_string(),
            " 8-jui-2001"
        );

        // time specifiers
        assert_eq!(dt.format_localized("%P", Locale::fr_BE).to_string(), "");
        assert_eq!(dt.format_localized("%p", Locale::fr_BE).to_string(), "");
        assert_eq!(
            dt.format_localized("%R", Locale::fr_BE).to_string(),
            "00:34"
        );
        assert_eq!(
            dt.format_localized("%T", Locale::fr_BE).to_string(),
            "00:34:60"
        );
        assert_eq!(
            dt.format_localized("%X", Locale::fr_BE).to_string(),
            "00:34:60"
        );
        assert_eq!(
            dt.format_localized("%r", Locale::fr_BE).to_string(),
            "00:34:60"
        );

        // date & time specifiers
        assert_eq!(
            dt.format_localized("%c", Locale::fr_BE).to_string(),
            "dim 08 jui 2001 00:34:60 +09:30"
        );

        let nd = NaiveDate::from_ymd_opt(2001, 7, 8).unwrap();

        // date specifiers
        assert_eq!(nd.format_localized("%b", Locale::de_DE).to_string(), "Jul");
        assert_eq!(nd.format_localized("%B", Locale::de_DE).to_string(), "Juli");
        assert_eq!(nd.format_localized("%h", Locale::de_DE).to_string(), "Jul");
        assert_eq!(nd.format_localized("%a", Locale::de_DE).to_string(), "So");
        assert_eq!(
            nd.format_localized("%A", Locale::de_DE).to_string(),
            "Sonntag"
        );
        assert_eq!(
            nd.format_localized("%D", Locale::de_DE).to_string(),
            "07/08/01"
        );
        assert_eq!(
            nd.format_localized("%x", Locale::de_DE).to_string(),
            "08.07.2001"
        );
        assert_eq!(
            nd.format_localized("%F", Locale::de_DE).to_string(),
            "2001-07-08"
        );
        assert_eq!(
            nd.format_localized("%v", Locale::de_DE).to_string(),
            " 8-Jul-2001"
        );
    }

    /// Ensure parsing a timestamp with the parse-only stftime formatter "%#z" does
    /// not cause a panic.
    ///
    /// See <https://github.com/chronotope/chrono/issues/1139>.
    #[test]
    #[cfg(feature = "alloc")]
    fn test_parse_only_timezone_offset_permissive_no_panic() {
        use crate::NaiveDate;
        use crate::{FixedOffset, TimeZone};
        use std::fmt::Write;

        let dt = FixedOffset::east_opt(34200)
            .unwrap()
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2001, 7, 8)
                    .unwrap()
                    .and_hms_nano_opt(0, 34, 59, 1_026_490_708)
                    .unwrap(),
            )
            .unwrap();

        let mut buf = String::new();
        let _ = write!(buf, "{}", dt.format("%#z")).expect_err("parse-only formatter should fail");
    }

    #[test]
    #[cfg(all(feature = "unstable-locales", feature = "alloc"))]
    fn test_strftime_localized_korean() {
        let dt = FixedOffset::east_opt(34200)
            .unwrap()
            .with_ymd_and_hms(2001, 7, 8, 0, 34, 59)
            .unwrap()
            .with_nanosecond(1_026_490_708)
            .unwrap();

        // date specifiers
        assert_eq!(dt.format_localized("%b", Locale::ko_KR).to_string(), " 7월");
        assert_eq!(dt.format_localized("%B", Locale::ko_KR).to_string(), "7월");
        assert_eq!(dt.format_localized("%h", Locale::ko_KR).to_string(), " 7월");
        assert_eq!(dt.format_localized("%a", Locale::ko_KR).to_string(), "일");
        assert_eq!(
            dt.format_localized("%A", Locale::ko_KR).to_string(),
            "일요일"
        );
        assert_eq!(
            dt.format_localized("%D", Locale::ko_KR).to_string(),
            "07/08/01"
        );
        assert_eq!(
            dt.format_localized("%x", Locale::ko_KR).to_string(),
            "2001년 07월 08일"
        );
        assert_eq!(
            dt.format_localized("%F", Locale::ko_KR).to_string(),
            "2001-07-08"
        );
        assert_eq!(
            dt.format_localized("%v", Locale::ko_KR).to_string(),
            " 8- 7월-2001"
        );
        assert_eq!(
            dt.format_localized("%r", Locale::ko_KR).to_string(),
            "오전 12시 34분 60초"
        );

        // date & time specifiers
        assert_eq!(
            dt.format_localized("%c", Locale::ko_KR).to_string(),
            "2001년 07월 08일 (일) 오전 12시 34분 60초"
        );
    }

    #[test]
    #[cfg(all(feature = "unstable-locales", feature = "alloc"))]
    fn test_strftime_localized_japanese() {
        let dt = FixedOffset::east_opt(34200)
            .unwrap()
            .with_ymd_and_hms(2001, 7, 8, 0, 34, 59)
            .unwrap()
            .with_nanosecond(1_026_490_708)
            .unwrap();

        // date specifiers
        assert_eq!(dt.format_localized("%b", Locale::ja_JP).to_string(), " 7月");
        assert_eq!(dt.format_localized("%B", Locale::ja_JP).to_string(), "7月");
        assert_eq!(dt.format_localized("%h", Locale::ja_JP).to_string(), " 7月");
        assert_eq!(dt.format_localized("%a", Locale::ja_JP).to_string(), "日");
        assert_eq!(
            dt.format_localized("%A", Locale::ja_JP).to_string(),
            "日曜日"
        );
        assert_eq!(
            dt.format_localized("%D", Locale::ja_JP).to_string(),
            "07/08/01"
        );
        assert_eq!(
            dt.format_localized("%x", Locale::ja_JP).to_string(),
            "2001年07月08日"
        );
        assert_eq!(
            dt.format_localized("%F", Locale::ja_JP).to_string(),
            "2001-07-08"
        );
        assert_eq!(
            dt.format_localized("%v", Locale::ja_JP).to_string(),
            " 8- 7月-2001"
        );
        assert_eq!(
            dt.format_localized("%r", Locale::ja_JP).to_string(),
            "午前12時34分60秒"
        );

        // date & time specifiers
        assert_eq!(
            dt.format_localized("%c", Locale::ja_JP).to_string(),
            "2001年07月08日 00時34分60秒"
        );
    }

    #[test]
    #[cfg(all(feature = "unstable-locales", feature = "alloc"))]
    fn test_strftime_localized_time() {
        let dt1 = Utc.with_ymd_and_hms(2024, 2, 9, 6, 54, 32).unwrap();
        let dt2 = Utc.with_ymd_and_hms(2024, 2, 9, 18, 54, 32).unwrap();
        // Some of these locales gave issues before pure-rust-locales 0.8.0 with chrono 0.4.27+
        assert_eq!(
            dt1.format_localized("%X", Locale::nl_NL).to_string(),
            "06:54:32"
        );
        assert_eq!(
            dt2.format_localized("%X", Locale::nl_NL).to_string(),
            "18:54:32"
        );
        assert_eq!(
            dt1.format_localized("%X", Locale::en_US).to_string(),
            "06:54:32 AM"
        );
        assert_eq!(
            dt2.format_localized("%X", Locale::en_US).to_string(),
            "06:54:32 PM"
        );
        assert_eq!(
            dt1.format_localized("%X", Locale::hy_AM).to_string(),
            "06:54:32"
        );
        assert_eq!(
            dt2.format_localized("%X", Locale::hy_AM).to_string(),
            "18:54:32"
        );
        assert_eq!(
            dt1.format_localized("%X", Locale::chr_US).to_string(),
            "06:54:32 ᏌᎾᎴ"
        );
        assert_eq!(
            dt2.format_localized("%X", Locale::chr_US).to_string(),
            "06:54:32 ᏒᎯᏱᎢᏗᏢ"
        );
    }

    #[test]
    #[cfg(all(feature = "unstable-locales", target_pointer_width = "64"))]
    fn test_type_sizes() {
        use core::mem::size_of;
        assert_eq!(size_of::<Item>(), 24);
        assert_eq!(size_of::<StrftimeItems>(), 56);
        assert_eq!(size_of::<Locale>(), 2);
    }

    #[test]
    #[cfg(all(feature = "unstable-locales", target_pointer_width = "32"))]
    fn test_type_sizes() {
        use core::mem::size_of;
        assert_eq!(size_of::<Item>(), 12);
        assert_eq!(size_of::<StrftimeItems>(), 28);
        assert_eq!(size_of::<Locale>(), 2);
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_strftime_parse() {
        let fmt_str = StrftimeItems::new("%Y-%m-%dT%H:%M:%S%z");
        let fmt_items = fmt_str.parse().unwrap();
        let dt = Utc.with_ymd_and_hms(2014, 5, 7, 12, 34, 56).unwrap();
        assert_eq!(
            &dt.format_with_items(fmt_items.iter()).to_string(),
            "2014-05-07T12:34:56+0000"
        );
    }

    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_strftime_parse_lenient() {
        let fmt_str = StrftimeItems::new_lenient("%Y-%m-%dT%H:%M:%S%z%Q%.2f%%%");
        let fmt_items = fmt_str.parse().unwrap();
        let dt = Utc.with_ymd_and_hms(2014, 5, 7, 12, 34, 56).unwrap();
        assert_eq!(
            &dt.format_with_items(fmt_items.iter()).to_string(),
            "2014-05-07T12:34:56+0000%Q%.2f%%"
        );
    }

    /// Regression test for https://github.com/chronotope/chrono/issues/1725
    #[test]
    #[cfg(any(feature = "alloc", feature = "std"))]
    fn test_finite() {
        let mut i = 0;
        for item in StrftimeItems::new("%2f") {
            println!("{:?}", item);
            i += 1;
            if i > 10 {
                panic!("infinite loop");
            }
        }
    }
