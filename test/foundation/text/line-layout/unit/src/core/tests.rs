
    use super::*;
    use alloc::format;

    #[cfg(feature = "unicode-width")]
    use unicode_width::UnicodeWidthChar;

    #[test]
    fn skip_ansi_escape_sequence_works() {
        let blue_text = "\u{1b}[34mHello\u{1b}[0m";
        let mut chars = blue_text.chars();
        let ch = chars.next().unwrap();
        assert!(skip_ansi_escape_sequence(ch, &mut chars));
        assert_eq!(chars.next(), Some('H'));
    }

    #[test]
    fn emojis_have_correct_width() {
        use unic_emoji_char::is_emoji;

        // Emojis in the Basic Latin (ASCII) and Latin-1 Supplement
        // blocks all have a width of 1 column. This includes
        // characters such as '#' and '©'.
        for ch in '\u{1}'..'\u{FF}' {
            if is_emoji(ch) {
                let desc = format!("{:?} U+{:04X}", ch, ch as u32);

                #[cfg(feature = "unicode-width")]
                assert_eq!(ch.width().unwrap(), 1, "char: {}", desc);

                #[cfg(not(feature = "unicode-width"))]
                assert_eq!(ch_width(ch), 1, "char: {}", desc);
            }
        }

        // Emojis in the remaining blocks of the Basic Multilingual
        // Plane (BMP), in the Supplementary Multilingual Plane (SMP),
        // and in the Supplementary Ideographic Plane (SIP), are all 1
        // or 2 columns wide when unicode-width is used, and always 2
        // columns wide otherwise. This includes all of our favorite
        // emojis such as 😊.
        for ch in '\u{FF}'..'\u{2FFFF}' {
            if is_emoji(ch) {
                let desc = format!("{:?} U+{:04X}", ch, ch as u32);

                #[cfg(feature = "unicode-width")]
                assert!(ch.width().unwrap() <= 2, "char: {}", desc);

                #[cfg(not(feature = "unicode-width"))]
                assert_eq!(ch_width(ch), 2, "char: {}", desc);
            }
        }

        // The remaining planes contain almost no assigned code points
        // and thus also no emojis.
    }

    #[test]
    fn display_width_works() {
        assert_eq!("Café Plain".len(), 11); // “é” is two bytes
        assert_eq!(display_width("Café Plain"), 10);
        assert_eq!(display_width("\u{1b}[31mCafé Rouge\u{1b}[0m"), 10);
        assert_eq!(
            display_width("\x1b]8;;http://example.com\x1b\\This is a link\x1b]8;;\x1b\\"),
            14
        );
    }

    #[test]
    fn display_width_narrow_emojis() {
        #[cfg(feature = "unicode-width")]
        assert_eq!(display_width("⁉"), 1);

        // The ⁉ character is above DOUBLE_WIDTH_CUTOFF.
        #[cfg(not(feature = "unicode-width"))]
        assert_eq!(display_width("⁉"), 2);
    }

    #[test]
    fn display_width_narrow_emojis_variant_selector() {
        #[cfg(feature = "unicode-width")]
        assert_eq!(display_width("⁉\u{fe0f}"), 1);

        // The variant selector-16 is also counted.
        #[cfg(not(feature = "unicode-width"))]
        assert_eq!(display_width("⁉\u{fe0f}"), 4);
    }

    #[test]
    fn display_width_emojis() {
        assert_eq!(display_width("😂😭🥺🤣✨😍🙏🥰😊🔥"), 20);
    }
