#[derive(Debug, Clone)]
pub enum BaselineKind {
    Year(u32),
    LegacyYear(u32),
    InvalidYear,
    NewlyAvailable,
    NewlyAvailableOnDate,
    WidelyAvailable,
    WidelyAvailableOnDate { year: i64, month: u32, day: u32 },
}

#[derive(Debug, Clone)]
pub enum QueryAtom<'a> {
    Last { count: u16, major: bool, name: Option<&'a str> },
    Unreleased(Option<&'a str>),
    Years(f64),
    Since { year: i32, month: u32, day: u32 },
    Percentage { comparator: Comparator, popularity: f32, stats: Stats<'a> },
    Cover { coverage: f32, stats: Stats<'a> },
    Supports(&'a str, Option<SupportKind>),
    Electron(VersionRange<'a>),
    Node(VersionRange<'a>),
    Browser(&'a str, VersionRange<'a>),
    Baseline { kind: BaselineKind, downstream: bool, kaios: bool },
    FirefoxESR,
    OperaMini,
    CurrentNode,
    MaintainedNode,
    Phantom(bool),
    Defaults,
    Dead,
    Unknown(&'a str),
}

#[derive(Debug, Clone)]
pub enum Stats<'a> {
    Global,
    Region(&'a str),
}

#[derive(Debug, Clone, Copy)]
pub enum SupportKind {
    Fully,
    Partially,
}

#[derive(Debug, Clone, Copy)]
pub enum Comparator {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

impl Comparator {
    pub fn compare<T: PartialOrd>(self, value: T, target: T) -> bool {
        match self {
            Self::Greater => value > target,
            Self::GreaterOrEqual => value >= target,
            Self::Less => value < target,
            Self::LessOrEqual => value <= target,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VersionRange<'a> {
    Bounded(&'a str, &'a str),
    Unbounded(Comparator, &'a str),
    Accurate(&'a str),
}

#[derive(Debug)]
pub struct SingleQuery<'a> {
    pub(crate) raw: &'a str,
    pub(crate) atom: QueryAtom<'a>,
    pub(crate) negated: bool,
    pub(crate) is_and: bool,
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    #[inline]
    fn new(input: &'a str) -> Self {
        Self { input, bytes: input.as_bytes(), pos: 0 }
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    #[inline]
    fn peek(&self) -> u8 {
        self.bytes[self.pos]
    }

    #[inline]
    fn slice(&self, start: usize, end: usize) -> &'a str {
        &self.input[start..end]
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while self.pos < self.bytes.len() {
            match self.peek() {
                b' ' | b'\t' => self.pos += 1,
                _ => break,
            }
        }
    }

    #[inline]
    fn skip_whitespace1(&mut self) -> bool {
        let start = self.pos;
        self.skip_whitespace();
        self.pos > start
    }

    #[inline]
    fn eat(&mut self, b: u8) -> bool {
        if self.pos < self.bytes.len() && self.peek() == b {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    fn match_keyword(&mut self, kw: &[u8]) -> bool {
        let end = self.pos + kw.len();
        if end > self.bytes.len() {
            return false;
        }
        let slice = &self.bytes[self.pos..end];
        if !slice.eq_ignore_ascii_case(kw) {
            return false;
        }
        // Word boundary check
        if end < self.bytes.len() {
            let next = self.bytes[end];
            if next.is_ascii_alphanumeric() || next == b'_' {
                return false;
            }
        }
        self.pos = end;
        true
    }

    #[inline]
    fn match_bytes(&mut self, s: &[u8]) -> bool {
        let end = self.pos + s.len();
        if end <= self.bytes.len() {
            let slice = &self.bytes[self.pos..end];
            if slice == s {
                self.pos = end;
                return true;
            }
        }
        false
    }

    /// Match a keyword with an optional trailing `s` (e.g. `version`/`versions`).
    #[inline]
    fn match_plural_keyword(&mut self, kw: &[u8]) -> bool {
        let end = self.pos + kw.len();
        if end > self.bytes.len() {
            return false;
        }
        let slice = &self.bytes[self.pos..end];
        if !slice.eq_ignore_ascii_case(kw) {
            return false;
        }
        self.pos = end;
        if self.pos < self.bytes.len() && matches!(self.peek(), b's' | b'S') {
            self.pos += 1;
        }
        // Word boundary
        self.pos >= self.bytes.len() || {
            let b = self.peek();
            !b.is_ascii_alphanumeric() && b != b'_'
        }
    }

    #[inline]
    fn match_version_keyword(&mut self) -> bool {
        self.match_plural_keyword(b"version")
    }

    #[inline]
    fn match_year_keyword(&mut self) -> bool {
        self.match_plural_keyword(b"year")
    }

    /// Parse unsigned integer directly from bytes
    #[inline]
    fn parse_u16(&mut self) -> Option<u16> {
        if self.pos >= self.bytes.len() || !self.peek().is_ascii_digit() {
            return None;
        }
        let mut n: u16 = 0;
        while self.pos < self.bytes.len() {
            let b = self.peek();
            if !b.is_ascii_digit() {
                break;
            }
            n = n.checked_mul(10)?.checked_add(u16::from(b - b'0'))?;
            self.pos += 1;
        }
        Some(n)
    }

    #[inline]
    fn parse_u32(&mut self) -> Option<u32> {
        if self.pos >= self.bytes.len() || !self.peek().is_ascii_digit() {
            return None;
        }
        let mut n: u32 = 0;
        while self.pos < self.bytes.len() {
            let b = self.peek();
            if !b.is_ascii_digit() {
                break;
            }
            n = n.checked_mul(10)?.checked_add(u32::from(b - b'0'))?;
            self.pos += 1;
        }
        Some(n)
    }

    #[inline]
    fn parse_exact_digits(&mut self, count: usize) -> Option<u32> {
        let start = self.pos;
        let mut n = 0u32;
        for _ in 0..count {
            let Some(&byte) = self.bytes.get(self.pos) else {
                self.pos = start;
                return None;
            };
            if !byte.is_ascii_digit() {
                self.pos = start;
                return None;
            }
            n = n * 10 + u32::from(byte - b'0');
            self.pos += 1;
        }
        Some(n)
    }

    #[inline]
    fn parse_i32(&mut self) -> Option<i32> {
        let neg = self.eat(b'-');
        if !neg {
            self.eat(b'+');
        }
        let n = i64::from(self.parse_u32()?);
        i32::try_from(if neg { -n } else { n }).ok()
    }

    /// Scan an optionally signed decimal number (e.g. `-1`, `2.5`, `.5`).
    #[inline]
    fn scan_number(&mut self) -> Option<&'a str> {
        let start = self.pos;
        let _ = self.eat(b'-') || self.eat(b'+');
        while self.pos < self.bytes.len() && self.peek().is_ascii_digit() {
            self.pos += 1;
        }
        if self.eat(b'.') {
            while self.pos < self.bytes.len() && self.peek().is_ascii_digit() {
                self.pos += 1;
            }
        }
        if self.pos > start { Some(self.slice(start, self.pos)) } else { None }
    }

    #[inline]
    fn parse_float(&mut self) -> Option<f32> {
        self.scan_number()?.parse().ok()
    }

    #[inline]
    fn parse_double(&mut self) -> Option<f64> {
        self.scan_number()?.parse().ok()
    }

    #[inline]
    fn parse_version(&mut self) -> Option<&'a str> {
        let start = self.pos;
        // Version must start with a digit (reject leading dot like ".7")
        if self.pos >= self.bytes.len() || !self.peek().is_ascii_digit() {
            return None;
        }
        while self.pos < self.bytes.len() {
            let b = self.peek();
            if !b.is_ascii_digit() && b != b'.' {
                break;
            }
            self.pos += 1;
        }
        // Don't accept trailing dot (e.g., "2." is invalid)
        if self.pos > start && self.bytes[self.pos - 1] == b'.' {
            self.pos -= 1;
        }
        if self.pos > start { Some(self.slice(start, self.pos)) } else { None }
    }

    #[inline]
    fn parse_identifier(&mut self) -> Option<&'a str> {
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.peek();
            if !b.is_ascii_alphabetic() && b != b'_' {
                break;
            }
            self.pos += 1;
        }
        if self.pos > start { Some(self.slice(start, self.pos)) } else { None }
    }

    /// Parse a feature name for `supports` queries (alphanumeric and `-`).
    #[inline]
    fn parse_feature_name(&mut self) -> Option<&'a str> {
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.peek();
            if !b.is_ascii_alphanumeric() && b != b'-' {
                break;
            }
            self.pos += 1;
        }
        if self.pos > start { Some(self.slice(start, self.pos)) } else { None }
    }

    #[inline]
    fn parse_comparator(&mut self) -> Option<Comparator> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        match self.peek() {
            b'<' => {
                self.pos += 1;
                Some(if self.eat(b'=') { Comparator::LessOrEqual } else { Comparator::Less })
            }
            b'>' => {
                self.pos += 1;
                Some(if self.eat(b'=') { Comparator::GreaterOrEqual } else { Comparator::Greater })
            }
            _ => None,
        }
    }

    #[inline]
    fn parse_region(&mut self) -> Option<&'a str> {
        let start = self.pos;
        // Optional "alt-" prefix
        if self.pos + 4 <= self.bytes.len() {
            let slice = &self.bytes[self.pos..self.pos + 4];
            if slice.eq_ignore_ascii_case(b"alt-") {
                self.pos += 4;
            }
        }
        // Exactly 2 alphabetic
        if self.pos + 2 <= self.bytes.len() {
            let b1 = self.bytes[self.pos];
            let b2 = self.bytes[self.pos + 1];
            if b1.is_ascii_alphabetic() && b2.is_ascii_alphabetic() {
                self.pos += 2;
                return Some(self.slice(start, self.pos));
            }
        }
        self.pos = start;
        None
    }

    #[inline]
    fn parse_version_range(&mut self) -> Option<VersionRange<'a>> {
        let start = self.pos;

        // Try bounded: " 1.0 - 2.0"
        if self.skip_whitespace1()
            && let Some(from) = self.parse_version()
        {
            self.skip_whitespace();
            if self.eat(b'-') {
                self.skip_whitespace();
                if let Some(to) = self.parse_version() {
                    return Some(VersionRange::Bounded(from, to));
                }
            }
        }
        self.pos = start;

        // Try unbounded: ">= 1.0"
        self.skip_whitespace();
        if let Some(cmp) = self.parse_comparator() {
            self.skip_whitespace();
            if let Some(ver) = self.parse_version() {
                return Some(VersionRange::Unbounded(cmp, ver));
            }
        }
        self.pos = start;

        // Try accurate: " 1.0"
        if self.skip_whitespace1()
            && let Some(ver) = self.parse_version()
        {
            return Some(VersionRange::Accurate(ver));
        }
        self.pos = start;
        None
    }

    // =========================================================================
    // Query atom parsers - organized for first-byte dispatch
    // =========================================================================

    fn parse_last_or_years(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"last") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }

        // Could be "last N versions" or "last N years"
        let before_num = self.pos;

        // Try years first (has fractional)
        if let Some(years) = self.parse_double()
            && self.skip_whitespace1()
            && self.match_year_keyword()
        {
            return Some(QueryAtom::Years(years));
        }
        self.pos = before_num;

        // Try last N versions
        let Some(count) = self.parse_u16() else {
            self.pos = start;
            return None;
        };
        if !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }

        // Optional browser name
        let before_name = self.pos;
        let name = self.parse_identifier().filter(|s| {
            !s.eq_ignore_ascii_case("version")
                && !s.eq_ignore_ascii_case("versions")
                && !s.eq_ignore_ascii_case("major")
                && !s.eq_ignore_ascii_case("year")
                && !s.eq_ignore_ascii_case("years")
        });

        if name.is_some() {
            if !self.skip_whitespace1() {
                self.pos = start;
                return None;
            }
        } else {
            self.pos = before_name;
        }

        let major = if self.match_keyword(b"major") {
            if !self.skip_whitespace1() {
                self.pos = start;
                return None;
            }
            true
        } else {
            false
        };

        if !self.match_version_keyword() {
            self.pos = start;
            return None;
        }

        Some(QueryAtom::Last { count, major, name })
    }

    fn parse_unreleased(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"unreleased") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }

        let before_name = self.pos;
        let name = self
            .parse_identifier()
            .filter(|s| !s.eq_ignore_ascii_case("version") && !s.eq_ignore_ascii_case("versions"));

        if name.is_some() {
            if !self.skip_whitespace1() {
                self.pos = start;
                return None;
            }
        } else {
            self.pos = before_name;
        }

        if !self.match_version_keyword() {
            self.pos = start;
            return None;
        }
        Some(QueryAtom::Unreleased(name))
    }

    fn parse_since_or_supports(&mut self) -> Option<QueryAtom<'a>> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        // Peek second char to disambiguate
        if self.pos + 1 < self.bytes.len() {
            match self.bytes[self.pos + 1] {
                b'i' | b'I' => return self.parse_since(),
                b'u' | b'U' => return self.parse_supports_only(),
                _ => return self.parse_since().or_else(|| self.parse_supports_only()),
            }
        }
        self.parse_since().or_else(|| self.parse_supports_only())
    }

    fn parse_since(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        let result = (|| {
            self.match_keyword(b"since").then_some(())?;
            // Exactly one whitespace character, like the upstream `since` regex.
            (self.eat(b' ') || self.eat(b'\t')).then_some(())?;

            let year = self.parse_i32()?;
            let month = if self.eat(b'-') { self.parse_u32()? } else { 1 };
            let day = if self.eat(b'-') { self.parse_u32()? } else { 1 };

            Some(QueryAtom::Since { year, month, day })
        })();
        if result.is_none() {
            self.pos = start;
        }
        result
    }

    fn parse_supports_only(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"supports") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }

        let Some(feature) = self.parse_feature_name() else {
            self.pos = start;
            return None;
        };
        Some(QueryAtom::Supports(feature, None))
    }

    fn parse_cover_or_current(&mut self) -> Option<QueryAtom<'a>> {
        if self.pos + 1 >= self.bytes.len() {
            return None;
        }
        match self.bytes[self.pos + 1] {
            b'o' | b'O' => self.parse_cover(),
            b'u' | b'U' => self.parse_current_node(),
            _ => self.parse_cover().or_else(|| self.parse_current_node()),
        }
    }

    fn parse_cover(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"cover") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }
        let Some(coverage) = self.parse_float() else {
            self.pos = start;
            return None;
        };
        if !self.eat(b'%') {
            self.pos = start;
            return None;
        }

        let stats = if self.skip_whitespace1() && self.match_keyword(b"in") {
            if !self.skip_whitespace1() {
                self.pos = start;
                return None;
            }
            let Some(region) = self.parse_region() else {
                self.pos = start;
                return None;
            };
            Stats::Region(region)
        } else {
            Stats::Global
        };

        Some(QueryAtom::Cover { coverage, stats })
    }

    fn parse_percentage(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        let comparator = self.parse_comparator()?;
        self.skip_whitespace();
        let Some(popularity) = self.parse_float() else {
            self.pos = start;
            return None;
        };
        if !self.eat(b'%') {
            self.pos = start;
            return None;
        }

        let stats = if self.skip_whitespace1() && self.match_keyword(b"in") {
            if !self.skip_whitespace1() {
                self.pos = start;
                return None;
            }
            let Some(region) = self.parse_region() else {
                self.pos = start;
                return None;
            };
            Stats::Region(region)
        } else {
            Stats::Global
        };

        Some(QueryAtom::Percentage { comparator, popularity, stats })
    }

    fn parse_electron(&mut self) -> Option<QueryAtom<'a>> {
        if !self.match_keyword(b"electron") {
            return None;
        }
        self.parse_version_range().map(QueryAtom::Electron)
    }

    fn parse_node(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"node") {
            return None;
        }
        if let Some(range) = self.parse_version_range() {
            return Some(QueryAtom::Node(range));
        }
        self.pos = start;
        None
    }

    fn parse_firefox_or_fully(&mut self) -> Option<QueryAtom<'a>> {
        if self.pos + 1 >= self.bytes.len() {
            return None;
        }
        match self.bytes[self.pos + 1] {
            b'i' | b'I' | b'x' | b'X' | b'f' | b'F' => self.parse_firefox_esr(),
            b'u' | b'U' => self.parse_fully_supports(),
            _ => self.parse_firefox_esr().or_else(|| self.parse_fully_supports()),
        }
    }

    fn parse_firefox_esr(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"firefox")
            && !self.match_keyword(b"ff")
            && !self.match_keyword(b"fx")
        {
            return None;
        }
        if !self.skip_whitespace1() || !self.match_keyword(b"esr") {
            self.pos = start;
            return None;
        }
        Some(QueryAtom::FirefoxESR)
    }

    fn parse_fully_supports(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"fully") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }
        if !self.match_keyword(b"supports") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }
        let Some(feature) = self.parse_feature_name() else {
            self.pos = start;
            return None;
        };
        Some(QueryAtom::Supports(feature, Some(SupportKind::Fully)))
    }

    fn parse_operamini(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"operamini") && !self.match_keyword(b"op_mini") {
            return None;
        }
        if !self.skip_whitespace1() || !self.match_keyword(b"all") {
            self.pos = start;
            return None;
        }
        Some(QueryAtom::OperaMini)
    }

    fn parse_current_node(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"current")
            || !self.skip_whitespace1()
            || !self.match_keyword(b"node")
        {
            self.pos = start;
            return None;
        }
        Some(QueryAtom::CurrentNode)
    }

    fn parse_maintained_node(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"maintained")
            || !self.skip_whitespace1()
            || !self.match_keyword(b"node")
            || !self.skip_whitespace1()
            || !self.match_keyword(b"versions")
        {
            self.pos = start;
            return None;
        }
        Some(QueryAtom::MaintainedNode)
    }

    fn parse_phantom_or_partially(&mut self) -> Option<QueryAtom<'a>> {
        if self.pos + 1 >= self.bytes.len() {
            return None;
        }
        match self.bytes[self.pos + 1] {
            b'h' | b'H' => self.parse_phantom(),
            b'a' | b'A' => self.parse_partially_supports(),
            _ => self.parse_phantom().or_else(|| self.parse_partially_supports()),
        }
    }

    fn parse_phantom(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"phantomjs") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }
        if self.match_bytes(b"2.1") {
            Some(QueryAtom::Phantom(true))
        } else if self.match_bytes(b"1.9") {
            Some(QueryAtom::Phantom(false))
        } else {
            self.pos = start;
            None
        }
    }

    fn parse_partially_supports(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        if !self.match_keyword(b"partially") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }
        if !self.match_keyword(b"supports") || !self.skip_whitespace1() {
            self.pos = start;
            return None;
        }
        let Some(feature) = self.parse_feature_name() else {
            self.pos = start;
            return None;
        };
        Some(QueryAtom::Supports(feature, Some(SupportKind::Partially)))
    }

    fn parse_defaults_or_dead(&mut self) -> Option<QueryAtom<'a>> {
        if self.pos + 2 >= self.bytes.len() {
            return None;
        }
        match self.bytes[self.pos + 2] {
            b'f' | b'F' => {
                if self.match_keyword(b"defaults") {
                    Some(QueryAtom::Defaults)
                } else {
                    None
                }
            }
            b'a' | b'A' => {
                if self.match_keyword(b"dead") {
                    Some(QueryAtom::Dead)
                } else {
                    None
                }
            }
            // "defaults" has 'f' as third char, "dead" has 'a' as third char
            // So this fallback can never match either keyword
            _ => None,
        }
    }

    fn parse_browser(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        let name = self.parse_identifier()?;

        if let Some(range) = self.parse_version_range() {
            return Some(QueryAtom::Browser(name, range));
        }

        if self.skip_whitespace1() && self.match_keyword(b"tp") {
            return Some(QueryAtom::Browser(name, VersionRange::Accurate("tp")));
        }

        self.pos = start;
        None
    }

    fn parse_baseline_date(&mut self) -> Option<(i64, u32, u32)> {
        let start = self.pos;
        let result = (|| {
            let year = i64::from(self.parse_exact_digits(4)?);
            self.eat(b'-').then_some(())?;
            let month = self.parse_exact_digits(2)?;
            self.eat(b'-').then_some(())?;
            let day = self.parse_exact_digits(2)?;
            Some((year, month, day))
        })();
        if result.is_none() {
            self.pos = start;
        }
        result
    }

    fn parse_baseline(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        let result = (|| {
            if !self.match_keyword(b"baseline") || !self.skip_whitespace1() {
                return None;
            }

            let kind = if self.pos < self.bytes.len() && self.peek().is_ascii_digit() {
                let start = self.pos;
                while self.pos < self.bytes.len() && self.peek().is_ascii_digit() {
                    self.pos += 1;
                }
                let raw_year = self.slice(start, self.pos);
                match raw_year.parse() {
                    Ok(year) if raw_year.len() == 4 => BaselineKind::Year(year),
                    Ok(year) => BaselineKind::LegacyYear(year),
                    Err(_) => BaselineKind::InvalidYear,
                }
            } else {
                let newly = if self.match_keyword(b"newly") {
                    true
                } else if self.match_keyword(b"widely") {
                    false
                } else {
                    return None;
                };
                if !self.skip_whitespace1() || !self.match_keyword(b"available") {
                    return None;
                }

                let before_date = self.pos;
                let date = if self.skip_whitespace1() && self.match_keyword(b"on") {
                    if !self.skip_whitespace1() {
                        return None;
                    }
                    Some(self.parse_baseline_date()?)
                } else {
                    self.pos = before_date;
                    None
                };
                match (newly, date) {
                    (true, Some(_)) => BaselineKind::NewlyAvailableOnDate,
                    (true, None) => BaselineKind::NewlyAvailable,
                    (false, Some((year, month, day))) => {
                        BaselineKind::WidelyAvailableOnDate { year, month, day }
                    }
                    (false, None) => BaselineKind::WidelyAvailable,
                }
            };

            let before_downstream = self.pos;
            let downstream = self.skip_whitespace1()
                && self.match_keyword(b"with")
                && self.skip_whitespace1()
                && self.match_keyword(b"downstream");
            if !downstream {
                self.pos = before_downstream;
            }

            let before_kaios = self.pos;
            let kaios = self.skip_whitespace1()
                && self.match_keyword(b"including")
                && self.skip_whitespace1()
                && self.match_keyword(b"kaios");
            if !kaios {
                self.pos = before_kaios;
            }

            Some(QueryAtom::Baseline { kind, downstream, kaios })
        })();
        if result.is_none() {
            self.pos = start;
        }
        result
    }

    /// Dispatch to the appropriate parser based on first character
    #[inline]
    fn parse_query_atom(&mut self) -> Option<QueryAtom<'a>> {
        if self.pos >= self.bytes.len() {
            return None;
        }

        match self.peek() | 0x20 {
            // ASCII lowercase
            b'<' | b'>' => self.parse_percentage(),
            b'l' => self.parse_last_or_years().or_else(|| self.parse_browser()),
            b'u' => self.parse_unreleased().or_else(|| self.parse_browser()),
            b's' => self.parse_since_or_supports().or_else(|| self.parse_browser()),
            b'c' => self.parse_cover_or_current().or_else(|| self.parse_browser()),
            b'e' => self.parse_electron().or_else(|| self.parse_browser()),
            b'n' => self.parse_node().or_else(|| self.parse_browser()),
            b'f' => self.parse_firefox_or_fully().or_else(|| self.parse_browser()),
            b'o' => self.parse_operamini().or_else(|| self.parse_browser()),
            b'm' => self.parse_maintained_node().or_else(|| self.parse_browser()),
            b'p' => self.parse_phantom_or_partially().or_else(|| self.parse_browser()),
            b'b' => self.parse_baseline().or_else(|| self.parse_browser()),
            b'd' => self.parse_defaults_or_dead().or_else(|| self.parse_browser()),
            b'a'..=b'z' => self.parse_browser(),
            _ => None,
        }
    }

    #[inline]
    fn at_composition_operator(&self) -> bool {
        if self.pos >= self.bytes.len() {
            return false;
        }
        if self.peek() == b',' {
            return true;
        }
        if !matches!(self.peek(), b' ' | b'\t') {
            return false;
        }
        // Skip whitespace to check for "and"/"or"
        let mut i = self.pos;
        while i < self.bytes.len() && matches!(self.bytes[i], b' ' | b'\t') {
            i += 1;
        }
        let rest = &self.bytes[i..];
        (rest.len() >= 4
            && rest[..3].eq_ignore_ascii_case(b"and")
            && matches!(rest[3], b' ' | b'\t'))
            || (rest.len() >= 3
                && rest[..2].eq_ignore_ascii_case(b"or")
                && matches!(rest[2], b' ' | b'\t'))
    }

    #[inline]
    fn parse_composition_operator(&mut self) -> Option<bool> {
        self.skip_whitespace();

        if self.eat(b',') {
            self.skip_whitespace();
            return Some(false);
        }

        if self.match_keyword(b"and") && self.skip_whitespace1() {
            return Some(true);
        } else if self.match_keyword(b"or") && self.skip_whitespace1() {
            return Some(false);
        }
        None
    }

    fn parse_unknown(&mut self) -> Option<QueryAtom<'a>> {
        let start = self.pos;
        while !self.is_eof() && !self.at_composition_operator() {
            self.pos += 1;
        }
        if self.pos > start { Some(QueryAtom::Unknown(self.slice(start, self.pos))) } else { None }
    }

    fn parse_single_query_atom(&mut self) -> Option<(bool, QueryAtom<'a>, &'a str)> {
        let start = self.pos;

        let negated = if self.match_keyword(b"not") && self.skip_whitespace1() {
            true
        } else {
            self.pos = start;
            false
        };

        let atom = self.parse_query_atom().or_else(|| self.parse_unknown())?;
        Some((negated, atom, self.slice(start, self.pos)))
    }
}

pub fn parse_browserslist_query(input: &str) -> Result<Vec<SingleQuery<'_>>, &str> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(vec![]);
    }

    let mut parser = Parser::new(input);
    let mut queries = Vec::with_capacity(4);

    let (negated, atom, raw) = parser.parse_single_query_atom().ok_or(input)?;
    queries.push(SingleQuery { raw, atom, negated, is_and: false });

    while !parser.is_eof() {
        let Some(is_and) = parser.parse_composition_operator() else {
            return Err(parser.slice(parser.pos, parser.bytes.len()));
        };

        let (negated, atom, raw) =
            parser.parse_single_query_atom().ok_or(parser.slice(parser.pos, parser.bytes.len()))?;
        queries.push(SingleQuery { raw, atom, negated, is_and });
    }

    Ok(queries)
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/browser_targets/src/parser/tests.rs"]
#[cfg(test)]
mod tests;
