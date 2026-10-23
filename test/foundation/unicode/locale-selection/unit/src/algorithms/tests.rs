
    use super::*;
    use icu_locale_core::Locale;
    use icu_locale_core::preferences::LocalePreferences;
    use writeable::Writeable;

    struct TestCase {
        input: &'static str,
        requires_data: bool,
        // Note: The first entry in the chain is the normalized locale
        expected_language_chain: &'static [&'static str],
        expected_script_chain: &'static [&'static str],
        expected_region_chain: &'static [&'static str],
    }

    // TODO: Consider loading these from a JSON file
    const TEST_CASES: &[TestCase] = &[
        TestCase {
            input: "en-fonipa",
            requires_data: false,
            expected_language_chain: &["en-fonipa", "en"],
            expected_script_chain: &["en-fonipa", "en"],
            expected_region_chain: &["en-fonipa", "en"],
        },
        TestCase {
            input: "en-US-u-sd-usca",
            requires_data: false,
            expected_language_chain: &["en-US-u-sd-usca", "en-US", "en"],
            expected_script_chain: &["en-US-u-sd-usca", "en-US", "en"],
            expected_region_chain: &["en-US-u-sd-usca", "en-US", "und-US-u-sd-usca", "und-US"],
        },
        TestCase {
            input: "en-US-fonipa-u-sd-usca",
            requires_data: false,
            expected_language_chain: &[
                "en-US-fonipa-u-sd-usca",
                "en-US-fonipa",
                "en-US",
                "en-fonipa",
                "en",
            ],
            expected_script_chain: &[
                "en-US-fonipa-u-sd-usca",
                "en-US-fonipa",
                "en-US",
                "en-fonipa",
                "en",
            ],
            expected_region_chain: &[
                "en-US-fonipa-u-sd-usca",
                "en-US-fonipa",
                "en-US",
                "und-US-fonipa-u-sd-usca",
                "und-US-fonipa",
                "und-US",
            ],
        },
        TestCase {
            input: "en-fonipa",
            requires_data: true,
            expected_language_chain: &["en-fonipa", "en"],
            expected_script_chain: &["en-fonipa", "en", "und-Latn-fonipa", "und-Latn"],
            expected_region_chain: &["en-US-fonipa", "en-US", "und-US-fonipa", "und-US"],
        },
        TestCase {
            input: "en-Latn-fonipa",
            requires_data: true,
            expected_language_chain: &["en-fonipa", "en"],
            expected_script_chain: &["en-fonipa", "en", "und-Latn-fonipa", "und-Latn"],
            expected_region_chain: &["en-US-fonipa", "en-US", "und-US-fonipa", "und-US"],
        },
        TestCase {
            input: "en-Latn-US-u-sd-usca",
            requires_data: true,
            expected_language_chain: &["en-US-u-sd-usca", "en-US", "en"],
            expected_script_chain: &["en-US-u-sd-usca", "en-US", "en", "und-Latn"],
            expected_region_chain: &["en-US-u-sd-usca", "en-US", "und-US-u-sd-usca", "und-US"],
        },
        TestCase {
            input: "sr-ME",
            requires_data: true,
            expected_language_chain: &["sr-ME", "sr-Latn"],
            expected_script_chain: &["sr-ME", "sr-Latn", "und-Latn"],
            expected_region_chain: &["sr-ME", "und-ME"],
        },
        TestCase {
            input: "sr-Latn-ME",
            requires_data: true,
            expected_language_chain: &["sr-ME", "sr-Latn"],
            expected_script_chain: &["sr-ME", "sr-Latn", "und-Latn"],
            expected_region_chain: &["sr-ME", "und-ME"],
        },
        TestCase {
            input: "sr-Cyrl-ME",
            requires_data: true,
            expected_language_chain: &["sr-Cyrl-ME", "sr"],
            expected_script_chain: &["sr-Cyrl-ME", "sr", "und-Cyrl"],
            expected_region_chain: &["sr-Cyrl-ME", "und-ME"],
        },
        TestCase {
            input: "sr-ME-fonipa",
            requires_data: true,
            expected_language_chain: &["sr-ME-fonipa", "sr-ME", "sr-Latn-fonipa", "sr-Latn"],
            expected_script_chain: &[
                "sr-ME-fonipa",
                "sr-ME",
                "sr-Latn-fonipa",
                "sr-Latn",
                "und-Latn-fonipa",
                "und-Latn",
            ],
            expected_region_chain: &["sr-ME-fonipa", "sr-ME", "und-ME-fonipa", "und-ME"],
        },
        TestCase {
            input: "sr-RS",
            requires_data: true,
            expected_language_chain: &["sr-RS", "sr"],
            expected_script_chain: &["sr-RS", "sr", "und-Cyrl"],
            expected_region_chain: &["sr-RS", "und-RS"],
        },
        TestCase {
            input: "sr-Cyrl-RS",
            requires_data: true,
            expected_language_chain: &["sr-RS", "sr"],
            expected_script_chain: &["sr-RS", "sr", "und-Cyrl"],
            expected_region_chain: &["sr-RS", "und-RS"],
        },
        TestCase {
            input: "sr-Latn-RS",
            requires_data: true,
            expected_language_chain: &["sr-Latn-RS", "sr-Latn"],
            expected_script_chain: &["sr-Latn-RS", "sr-Latn", "und-Latn"],
            expected_region_chain: &["sr-Latn-RS", "und-RS"],
        },
        TestCase {
            input: "de-Latn-LI",
            requires_data: true,
            expected_language_chain: &["de-LI", "de"],
            expected_script_chain: &["de-LI", "de", "und-Latn"],
            expected_region_chain: &["de-LI", "und-LI"],
        },
        TestCase {
            input: "ca-ES-valencia",
            requires_data: true,
            expected_language_chain: &["ca-ES-valencia", "ca-ES", "ca-valencia", "ca"],
            expected_script_chain: &[
                "ca-ES-valencia",
                "ca-ES",
                "ca-valencia",
                "ca",
                "und-Latn-valencia",
                "und-Latn",
            ],
            expected_region_chain: &["ca-ES-valencia", "ca-ES", "und-ES-valencia", "und-ES"],
        },
        TestCase {
            input: "es-AR",
            requires_data: true,
            expected_language_chain: &["es-AR", "es-419", "es"],
            expected_script_chain: &["es-AR", "es-419", "es", "und-Latn"],
            expected_region_chain: &["es-AR", "und-AR"],
        },
        TestCase {
            input: "hi-IN",
            requires_data: true,
            expected_language_chain: &["hi-IN", "hi"],
            expected_script_chain: &["hi-IN", "hi", "und-Deva"],
            expected_region_chain: &["hi-IN", "und-IN"],
        },
        TestCase {
            input: "hi-Latn-IN",
            requires_data: true,
            expected_language_chain: &["hi-Latn-IN", "hi-Latn", "en-IN", "en-001", "en"],
            expected_script_chain: &["hi-Latn-IN", "hi-Latn", "en-IN", "en-001", "en", "und-Latn"],
            expected_region_chain: &["hi-Latn-IN", "und-IN"],
        },
        TestCase {
            input: "zh-CN",
            requires_data: true,
            // Note: "zh-Hans" is not reachable because it is the default script for "zh".
            // The fallback algorithm does not visit the language-script bundle when the
            // script is the default for the language
            expected_language_chain: &["zh-CN", "zh"],
            expected_script_chain: &["zh-CN", "zh", "und-Hans", "und-Hani"],
            expected_region_chain: &["zh-CN", "und-CN"],
        },
        TestCase {
            input: "zh-Hant-CN",
            requires_data: true,
            expected_language_chain: &["zh-Hant-CN", "zh-Hant"],
            expected_script_chain: &["zh-Hant-CN", "zh-Hant", "und-Hant", "und-Hani"],
            expected_region_chain: &["zh-Hant-CN", "und-CN"],
        },
        TestCase {
            input: "zh-TW",
            requires_data: true,
            expected_language_chain: &["zh-TW", "zh-Hant"],
            expected_script_chain: &["zh-TW", "zh-Hant", "und-Hant", "und-Hani"],
            expected_region_chain: &["zh-TW", "und-TW"],
        },
        TestCase {
            input: "zh-Hans-TW",
            requires_data: true,
            expected_language_chain: &["zh-Hans-TW", "zh"],
            expected_script_chain: &["zh-Hans-TW", "zh", "und-Hans", "und-Hani"],
            expected_region_chain: &["zh-Hans-TW", "und-TW"],
        },
        TestCase {
            input: "yue-HK",
            requires_data: true,
            expected_language_chain: &["yue-HK", "yue"],
            expected_script_chain: &["yue-HK", "yue", "und-Hant", "und-Hani"],
            expected_region_chain: &["yue-HK", "und-HK"],
        },
        TestCase {
            input: "yue-HK",
            requires_data: true,
            expected_language_chain: &["yue-HK", "yue"],
            expected_script_chain: &["yue-HK", "yue", "und-Hant", "und-Hani"],
            expected_region_chain: &["yue-HK", "und-HK"],
        },
        TestCase {
            input: "yue-CN",
            requires_data: true,
            expected_language_chain: &["yue-CN", "yue-Hans"],
            expected_script_chain: &["yue-CN", "yue-Hans", "und-Hans", "und-Hani"],
            expected_region_chain: &["yue-CN", "und-CN"],
        },
        TestCase {
            input: "az-Arab-IR",
            requires_data: true,
            expected_language_chain: &["az-IR", "az-Arab"],
            expected_script_chain: &["az-IR", "az-Arab", "und-Arab"],
            expected_region_chain: &["az-IR", "und-IR"],
        },
        TestCase {
            input: "az-IR",
            requires_data: true,
            expected_language_chain: &["az-IR", "az-Arab"],
            expected_script_chain: &["az-IR", "az-Arab", "und-Arab"],
            expected_region_chain: &["az-IR", "und-IR"],
        },
        TestCase {
            input: "az-Arab",
            requires_data: true,
            expected_language_chain: &["az-Arab"],
            expected_script_chain: &["az-Arab", "und-Arab"],
            expected_region_chain: &["az-IR", "und-IR"],
        },
    ];

    #[test]
    fn test_fallback() {
        let fallbacker_no_data = LocaleFallbacker::new_without_data();
        let fallbacker_no_data = fallbacker_no_data.as_borrowed();
        let fallbacker_with_data = LocaleFallbacker::new();
        for cas in TEST_CASES {
            let prefs = LocalePreferences::from(&cas.input.parse::<Locale>().unwrap());
            for (priority, data_locale, expected_chain) in [
                (
                    LocaleFallbackPriority::Language,
                    prefs.to_data_locale_language_priority(),
                    cas.expected_language_chain,
                ),
                (
                    LocaleFallbackPriority::Script,
                    prefs.to_data_locale_language_priority(),
                    cas.expected_script_chain,
                ),
                (
                    LocaleFallbackPriority::Region,
                    prefs.to_data_locale_region_priority(),
                    cas.expected_region_chain,
                ),
            ] {
                let mut config = LocaleFallbackConfig::default();
                config.priority = priority;
                let fallbacker = if cas.requires_data {
                    fallbacker_with_data
                } else {
                    fallbacker_no_data
                };
                let mut it = fallbacker.for_config(config).fallback_for(data_locale);
                let mut actual_chain = Vec::new();
                for i in 0..20 {
                    if i == 19 {
                        eprintln!("20 iterations reached!");
                    }
                    if it.get().is_unknown() {
                        break;
                    }
                    actual_chain.push(it.get().write_to_string().into_owned());
                    it.step();
                }
                assert_eq!(
                    expected_chain, &actual_chain,
                    "{:?} ({:?})",
                    cas.input, priority
                );
            }
        }
    }
