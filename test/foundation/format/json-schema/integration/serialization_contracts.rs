use blog_format_json_schema::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Serialize, Serializer, ser::SerializeMap};
use std::{borrow::Cow, collections::BTreeMap};

#[test]
fn fragment_pointers_preserve_invalid_percent_escapes() {
    let mut schema = json_schema!({
        "$defs": {
            "%+1": { "title": "literal" },
            "\u{1}": { "title": "control" },
        }
    });
    assert_eq!(schema.pointer("#/$defs/%+1").unwrap()["title"], "literal");
    assert_eq!(schema.pointer("#/$defs/%01").unwrap()["title"], "control");
    assert_eq!(schema.pointer("#/$defs/%25+1").unwrap()["title"], "literal");
    schema.pointer_mut("#/$defs/%+1").unwrap()["title"] = "changed".into();
    assert_eq!(schema.as_value()["$defs"]["%+1"]["title"], "changed");
    assert_eq!(schema.as_value()["$defs"]["\u{1}"]["title"], "control");
}

#[test]
fn collected_map_schema_rejects_invalid_json_keys() {
    let invalid = BTreeMap::from([([1u8, 2], 3u8)]);
    assert!(
        SchemaGenerator::default()
            .root_schema_for_value(&invalid)
            .is_err()
    );
    assert!(
        SchemaGenerator::default()
            .into_root_schema_for_value(&invalid)
            .is_err()
    );

    let valid = BTreeMap::from([(42u8, 3u8)]);
    let schema = SchemaGenerator::default()
        .into_root_schema_for_value(&valid)
        .unwrap();
    assert_eq!(schema.as_value()["examples"][0]["42"], 3);
}

enum CollectedValue {
    Unit,
    Failure,
}

impl Serialize for CollectedValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Unit => serializer.serialize_unit(),
            Self::Failure => Err(serde::ser::Error::custom("example serialization failed")),
        }
    }
}

struct UnrepresentableExample;

impl Serialize for UnrepresentableExample {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_map([
            ("first", CollectedValue::Unit),
            ("second", CollectedValue::Failure),
        ])
    }
}

#[test]
fn root_schema_generation_propagates_example_serialization_errors() {
    for result in [
        SchemaGenerator::default().root_schema_for_value(&UnrepresentableExample),
        SchemaGenerator::default().into_root_schema_for_value(&UnrepresentableExample),
    ] {
        assert_eq!(
            result.unwrap_err().to_string(),
            "example serialization failed"
        );
    }
}

struct Entries;

impl Serialize for Entries {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("", &1u8)?;
        map.serialize_entry("named", &2u8)?;
        map.end()
    }
}

#[test]
fn map_schema_preserves_empty_and_named_keys() {
    let schema = SchemaGenerator::default()
        .into_root_schema_for_value(&Entries)
        .unwrap();
    let properties = schema.as_value()["properties"].as_object().unwrap();
    assert_eq!(properties.len(), 2);
    assert_eq!(properties[""]["type"], "integer");
    assert_eq!(properties["named"]["type"], "integer");
}

struct ValueWithoutKey;

impl Serialize for ValueWithoutKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_value(&1u8)?;
        map.end()
    }
}

#[test]
#[should_panic(expected = "SerializeMap::serialize_key precedes serialize_value")]
fn broken_serialize_contract_does_not_invent_an_empty_key() {
    let _schema = SchemaGenerator::default().into_root_schema_for_value(&ValueWithoutKey);
}

struct EscapedSchemaName;

impl JsonSchema for EscapedSchemaName {
    fn schema_name() -> Cow<'static, str> {
        "\0\u{1}\u{2}\u{3}\u{4}\u{5}\u{6}\u{7}\u{8}\t\n\u{b}\u{c}\r\u{e}\u{f}~/%🚀".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({ "type": "string" })
    }
}

#[test]
fn schema_references_encode_control_bytes_as_two_hex_digits() {
    let mut generator = SchemaGenerator::default();
    let schema = generator.subschema_for::<EscapedSchemaName>();
    let reference = schema.as_value()["$ref"].as_str().unwrap();
    assert_eq!(
        reference,
        "#/$defs/%00%01%02%03%04%05%06%07%08%09%0A%0B%0C%0D%0E%0F~0~1%25%F0%9F%9A%80"
    );
    let root = json_schema!({ "$defs": generator.definitions() });
    assert_eq!(
        root.pointer(reference),
        generator
            .definitions()
            .get(EscapedSchemaName::schema_name().as_ref())
    );
}

struct SingleEntry<K>(K);

impl<K: Serialize> Serialize for SingleEntry<K> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.0, &1u8)?;
        map.end()
    }
}

#[test]
fn map_schema_preserves_escaped_string_keys() {
    for key in ["", "\"quoted\"", "line\n\t\0", "back\\slash", "你好🚀"] {
        let schema = SchemaGenerator::default()
            .into_root_schema_for_value(&SingleEntry(key))
            .unwrap();
        let properties = schema.as_value()["properties"].as_object().unwrap();
        assert_eq!(properties.len(), 1);
        assert_eq!(properties[key]["type"], "integer");
    }
}

#[test]
fn map_schema_uses_json_key_rules() {
    let numeric = SchemaGenerator::default()
        .into_root_schema_for_value(&SingleEntry(42u8))
        .unwrap();
    assert_eq!(numeric.as_value()["properties"]["42"]["type"], "integer");
    assert!(
        SchemaGenerator::default()
            .into_root_schema_for_value(&SingleEntry([1u8, 2]))
            .is_err()
    );
}
