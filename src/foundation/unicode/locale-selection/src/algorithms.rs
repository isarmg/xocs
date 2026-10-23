// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use super::LocaleFallbackPriority;
use ::icu_locale_core::subtags::{Language, Region, Script};

use super::*;

impl LocaleFallbackerWithConfig<'_> {
    pub(crate) fn normalize(&self, locale: &mut DataLocale, default_script: &mut Option<Script>) {
        // 0. If there is an invalid or trivial "sd" subtag, drop it
        if let Some(subdivision) = locale.subdivision.take()
            && let Some(region) = locale.region
            && subdivision
                .as_str()
                .starts_with(region.to_tinystr().to_ascii_lowercase().as_str())
            && !subdivision.as_str().ends_with("zzzz")
        {
            locale.subdivision = Some(subdivision);
        }
        let language = locale.language;
        // 1. Populate the region (required for region fallback only)
        if self.config.priority == LocaleFallbackPriority::Region && locale.region.is_none() {
            // 1a. First look for region based on language+script
            if let Some(script) = locale.script {
                locale.region = self
                    .likely_subtags
                    .language_script
                    .get(&(
                        language.to_tinystr().to_unvalidated(),
                        script.to_tinystr().to_unvalidated(),
                    ))
                    .copied();
            }
            // 1b. If that fails, try language only
            if locale.region.is_none() {
                locale.region = self
                    .likely_subtags
                    .language
                    .get_copied(&language.to_tinystr().to_unvalidated())
                    .map(|(_s, r)| r);
            }
        }
        // 2. Remove the script if it is implied by the other subtags
        if locale.script.is_some()
            || locale.region.is_some()
            || self.config.priority == LocaleFallbackPriority::Script
        {
            *default_script = locale
                .region
                .and_then(|region| {
                    self.likely_subtags.language_region.get_copied(&(
                        language.to_tinystr().to_unvalidated(),
                        region.to_tinystr().to_unvalidated(),
                    ))
                })
                .or_else(|| {
                    self.likely_subtags
                        .language
                        .get_copied(&language.to_tinystr().to_unvalidated())
                        .map(|(s, _r)| s)
                });
            if locale.script == *default_script {
                locale.script = None;
            }
        }
    }
}

impl LocaleFallbackIteratorInner<'_> {
    pub fn step(&mut self, locale: &mut DataLocale) {
        match self.config.priority {
            LocaleFallbackPriority::Language => self.step_language(locale),
            LocaleFallbackPriority::Script => self.step_script(locale),
            LocaleFallbackPriority::Region => self.step_region(locale),
            // This case should not normally happen, but `LocaleFallbackPriority` is non_exhaustive.
            // Make it go directly to `und`.
            _ => {
                debug_assert!(
                    false,
                    "Unknown LocaleFallbackPriority: {:?}",
                    self.config.priority
                );
                *locale = Default::default()
            }
        }
    }

    fn step_language(&mut self, locale: &mut DataLocale) {
        // 2. Remove the subdivision keyword
        if let Some(value) = locale.subdivision.take() {
            self.backup_subdivision = Some(value);
            return;
        }
        // 4. Remove variants
        if let Some(single_variant) = locale.variant.take() {
            self.backup_variant = Some(single_variant);
            return;
        }
        // 5. Check for parent override
        if let Some((language, script, region)) = self.get_explicit_parent(locale) {
            locale.language = language;
            locale.script = script;
            locale.region = region;
            locale.variant = self.backup_variant.take();
            return;
        }
        // 7. Remove region
        if let Some(_region) = locale.region {
            // note: self.backup_region is not used right now
            // 6. Add or remove the script subtag if necessary
            let language_implied_script = self
                .likely_subtags
                .language
                .get_copied(&locale.language.to_tinystr().to_unvalidated())
                .map(|(s, _r)| s);
            if language_implied_script != self.max_script {
                locale.script = self.max_script;
            } else {
                locale.script = None;
            }
            locale.region = None;
            locale.variant = self.backup_variant.take();
            return;
        }
        // 8. Remove language+script
        debug_assert!(!locale.language.is_unknown() || locale.script.is_some()); // don't call .step() on und
        locale.script = None;
        locale.language = Language::UNKNOWN;
    }

    fn step_region(&mut self, locale: &mut DataLocale) {
        // 2. Remove the subdivision keyword
        if let Some(value) = locale.subdivision.take() {
            self.backup_subdivision = Some(value);
            return;
        }
        // 4. Remove variants
        if let Some(variant) = locale.variant.take() {
            self.backup_variant = Some(variant);
            return;
        }
        // 5. Remove language+script
        if !locale.language.is_unknown() || locale.script.is_some() {
            locale.script = None;
            locale.language = Language::UNKNOWN;
            // Don't produce und-variant
            if locale.region.is_some() {
                locale.variant = self.backup_variant.take();
                locale.subdivision = self.backup_subdivision.take();
            }
            return;
        }
        // 6. Remove region
        debug_assert!(locale.region.is_some()); // don't call .step() on und
        locale.region = None;
    }

    fn step_script(&mut self, locale: &mut DataLocale) {
        // Remove the subdivision keyword
        if let Some(value) = locale.subdivision.take() {
            self.backup_subdivision = Some(value);
            return;
        }
        // Remove variants
        if let Some(variant) = locale.variant.take() {
            self.backup_variant = Some(variant);
            return;
        }
        // Check for parent override
        if let Some((language, script, region)) = self.get_explicit_parent(locale) {
            locale.language = language;
            locale.script = script;
            locale.region = region;
            locale.variant = self.backup_variant.take();
            return;
        }
        // Remove the region
        if let Some(region) = locale.region {
            self.backup_region = Some(region);
            let language_implied_script = self
                .likely_subtags
                .language
                .get_copied(&locale.language.to_tinystr().to_unvalidated())
                .map(|(s, _r)| s);
            if language_implied_script != self.max_script {
                locale.script = self.max_script;
            } else {
                locale.script = None;
            }
            locale.region = None;
            locale.variant = self.backup_variant.take();
            return;
        }

        // Remove the script if we have a language
        if !locale.language.is_unknown() {
            let language_implied_script = self
                .likely_subtags
                .language
                .get_copied(&locale.language.to_tinystr().to_unvalidated())
                .map(|(s, _r)| s);
            if locale.script.is_some() && language_implied_script == locale.script {
                locale.script = None;
                if let Some(region) = self.backup_region.take() {
                    locale.region = Some(region);
                    locale.subdivision = self.backup_subdivision.take();
                    locale.variant = self.backup_variant.take();
                }
                // needed if more fallback is added at the end
            } else {
                // 3. Remove the language and apply the maximized script
                locale.language = Language::UNKNOWN;
                locale.script = self.max_script;
                // Don't produce und-variant
                if locale.script.is_some() {
                    locale.variant = self.backup_variant.take();
                }
                // needed if more fallback is added at the end
            }
        } else if locale.script.is_some() {
            // note: UTS #35 wants us to apply "other associated scripts" now. ICU4C/J does not do this,
            // so we don't either. They would be found here if they are ever needed:
            // https://github.com/unicode-cldr/cldr-core/blob/master/supplemental/languageData.json

            // 6. Remove script
            locale.script = None;
        }
    }

    fn get_explicit_parent(
        &self,
        locale: &DataLocale,
    ) -> Option<(Language, Option<Script>, Option<Region>)> {
        self.parents
            .parents
            .get_copied_by(|uvstr| locale.strict_cmp(uvstr).reverse())
    }
}

#[path = "../../../../../test/foundation/unicode/locale-selection/unit/src/algorithms/tests.rs"]
#[cfg(test)]
mod tests;
