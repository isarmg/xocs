use crate::language_semantics::ToInt32;

pub trait StringSubstring {
    /// `String.prototype.substring ( start , end ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.substring>
    fn substring(&self, start: Option<f64>, end: Option<f64>) -> String;
}

impl StringSubstring for &str {
    #[expect(clippy::cast_sign_loss)]
    fn substring(&self, start: Option<f64>, end: Option<f64>) -> String {
        let start = start.map_or(0, |x| x.to_int_32().max(0) as usize);
        let end = end.map_or(usize::MAX, |x| x.to_int_32().max(0) as usize);
        let start = start.min(self.len());
        let end = end.min(self.len());
        if start > end {
            return String::new();
        }
        self.chars().skip(start).take(end - start).collect()
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/language_semantics/src/string_substring/test.rs"]
#[cfg(test)]
mod test;
