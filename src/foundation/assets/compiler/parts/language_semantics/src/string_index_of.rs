use crate::language_semantics::ToInt32;

pub trait StringIndexOf {
    /// `String.prototype.indexOf ( searchString [ , position ] )`
    /// <https://tc39.es/ecma262/#sec-string.prototype.indexof>
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize;
}

impl StringIndexOf for &str {
    #[expect(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn index_of(&self, search_value: Option<&str>, from_index: Option<f64>) -> isize {
        let from_index = from_index.map_or(0, |x| x.to_int_32().max(0)) as usize;
        let search_value = search_value.unwrap_or("undefined");
        let result = self.chars().skip(from_index).collect::<String>().find(search_value);
        result.map(|index| index + from_index).map_or(-1, |index| index as isize)
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/language_semantics/src/string_index_of/test.rs"]
#[cfg(test)]
mod test;
