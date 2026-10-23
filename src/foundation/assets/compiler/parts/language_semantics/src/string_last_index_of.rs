use crate::language_semantics::ToInt32;

pub trait StringLastIndexOf {
    /// `String.prototype.lastIndexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.lastindexof>
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringLastIndexOf for &str {
    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn last_index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let search_value = search_value.unwrap_or("undefined");
        let from_index =
            from_index.map_or(usize::MAX, |x| x.to_int_32().max(0) as usize + search_value.len());
        self.chars()
            .take(from_index)
            .collect::<String>()
            .rfind(search_value)
            .map_or(-1, |index| index as isize)
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/language_semantics/src/string_last_index_of/test.rs"]
#[cfg(test)]
mod test;
