use crate::CompactString;

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::AsStr for CompactString {
    fn as_str(&self) -> &str {
        self.as_str()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::length::HasBytes for CompactString {
    fn num_bytes(&self) -> usize {
        self.len()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::length::HasChars for CompactString {
    fn num_chars(&self) -> usize {
        self.chars().count()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::length::HasSimpleLength for CompactString {
    fn length(&self) -> usize {
        self.len()
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "garde")))]
impl garde::rules::length::HasUtf16CodeUnits for CompactString {
    fn num_code_units(&self) -> usize {
        self.encode_utf16().count()
    }
}

#[path = "../../../../../../test/foundation/collections/inline-string/unit/src/features/garde/tests.rs"]
#[cfg(test)]
mod tests;
