use napi_derive::napi;

// Source map output using the standard ignoreList field.
// See <https://tc39.es/ecma426/#sec-source-map-format>.
//
// ```
// export interface ExistingRawSourceMap {
//  file?: string;
//  mappings: string;
//  names: string[];
//  sourceRoot?: string;
//  sources: string[];
//  sourcesContent?: string[];
//  version: number;
//  ignoreList?: number[];
// }
// ```
#[napi(object)]
pub struct SourceMap {
    pub file: Option<String>,
    pub mappings: String,
    pub names: Vec<String>,
    pub source_root: Option<String>,
    pub sources: Vec<String>,
    pub sources_content: Option<Vec<String>>,
    pub version: u8,
    #[napi(js_name = "ignoreList")]
    pub ignore_list: Option<Vec<u32>>,
}

impl From<crate::source_map::SourceMap<'_>> for SourceMap {
    fn from(source_map: crate::source_map::SourceMap<'_>) -> Self {
        let json = source_map.to_json();
        Self {
            file: json.file,
            mappings: json.mappings,
            names: json.names,
            source_root: json.source_root,
            sources: json.sources,
            sources_content: json
                .sources_content
                .map(|content| content.into_iter().map(Option::unwrap_or_default).collect()),
            version: 3,
            ignore_list: json.ignore_list,
        }
    }
}

impl From<crate::source_map::OwnedSourceMap> for SourceMap {
    #[inline]
    fn from(source_map: crate::source_map::OwnedSourceMap) -> Self {
        Self::from(source_map.into_inner())
    }
}

#[path = "../../../../../../../test/foundation/assets/compiler/unit/parts/source_map/src/napi/tests.rs"]
#[cfg(test)]
mod tests;
