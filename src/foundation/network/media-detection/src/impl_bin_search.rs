use unicase::UniCase;

include!("mime_types.rs");

pub fn get_mime_types(ext: &str) -> Option<&'static [&'static str]> {
    MIME_TYPES
        .binary_search_by_key(&UniCase::new(ext), |(extension, _)| {
            UniCase::new(*extension)
        })
        .ok()
        .map(|index| MIME_TYPES[index].1)
}
