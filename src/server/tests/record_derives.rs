//! Real SQLite consumer checks for the repository's syntax-only record derive.
use sqlx::{Connection, FromRow};

#[derive(Debug, FromRow, PartialEq)]
#[sqlx(rename_all = "camelCase")]
struct Named<T: Clone> {
    user_id: i64,
    #[sqlx(rename = "name")]
    display_name: String,
    value: T,
    #[sqlx(default)]
    absent: String,
    #[sqlx(skip)]
    ignored: Vec<u8>,
    r#type: String,
}

#[derive(Debug, FromRow, PartialEq)]
struct Pair<R>(i64, R);

#[derive(Debug, FromRow, PartialEq)]
struct Reserved<__BlogRow0> {
    value: __BlogRow0,
}

#[derive(Debug, Default, FromRow, PartialEq)]
#[sqlx(default)]
struct InternalNames {
    __row: i64,
    __default: i64,
    missing: String,
    value: i64,
}

mod shadowed_prelude {
    // Caller names must not redirect the derive's default initialization.
    #[allow(dead_code)]
    struct Default;

    #[derive(Debug, sqlx::FromRow, PartialEq)]
    pub struct Record {
        #[sqlx(default)]
        pub missing: String,
        #[sqlx(skip)]
        pub skipped: Vec<u8>,
    }
}

#[derive(Debug, FromRow)]
struct Borrowed<'row> {
    name: &'row str,
}

#[derive(Debug, FromRow, PartialEq)]
struct Child {
    leaf: i64,
}

#[derive(Debug, PartialEq)]
struct Byte(u8);
impl TryFrom<i64> for Byte {
    type Error = std::num::TryFromIntError;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        u8::try_from(value).map(Self)
    }
}

#[derive(Debug, PartialEq)]
struct ConvertedChild(Child);
impl From<Child> for ConvertedChild {
    fn from(value: Child) -> Self {
        Self(value)
    }
}

#[derive(Debug, FromRow, PartialEq)]
struct Nested {
    #[sqlx(flatten)]
    child: Child,
    #[sqlx(flatten, try_from = "Child")]
    converted: ConvertedChild,
    #[sqlx(try_from = "i64")]
    byte: Byte,
}

#[derive(Debug, FromRow, PartialEq)]
#[sqlx(default)]
struct GenericDefault<T> {
    present: i64,
    missing: T,
}
impl Default for GenericDefault<i64> {
    fn default() -> Self {
        Self {
            present: -1,
            missing: 69,
        }
    }
}

#[derive(Debug, FromRow, PartialEq)]
struct JsonValues {
    #[sqlx(json)]
    items: Vec<String>,
    #[sqlx(json(nullable))]
    nullable: Option<Vec<i64>>,
    #[sqlx(json, try_from = "i64")]
    byte: Byte,
}

#[tokio::test]
async fn named_tuple_borrowed_and_generic_records_decode_actual_sqlite_rows() -> sqlx::Result<()> {
    let mut database = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
    let named = sqlx::query_as::<_, Named<i64>>(
        "SELECT 7 AS userId,'Ada' AS name,42 AS value,'record' AS type",
    )
    .fetch_one(&mut database)
    .await?;
    assert_eq!(
        named,
        Named {
            user_id: 7,
            display_name: "Ada".into(),
            value: 42,
            absent: String::new(),
            ignored: vec![],
            r#type: "record".into()
        }
    );
    let pair = sqlx::query_as::<_, Pair<String>>("SELECT 3,'three'")
        .fetch_one(&mut database)
        .await?;
    assert_eq!(pair, Pair(3, "three".into()));
    let reserved = sqlx::query_as::<_, Reserved<i64>>("SELECT 11 AS value")
        .fetch_one(&mut database)
        .await?;
    assert_eq!(reserved, Reserved { value: 11 });
    let internal =
        sqlx::query_as::<_, InternalNames>("SELECT 12 AS __row,13 AS __default,14 AS value")
            .fetch_one(&mut database)
            .await?;
    assert_eq!(
        internal,
        InternalNames {
            __row: 12,
            __default: 13,
            missing: String::new(),
            value: 14,
        }
    );
    let defaults = sqlx::query_as::<_, shadowed_prelude::Record>("SELECT 1")
        .fetch_one(&mut database)
        .await?;
    assert!(defaults.missing.is_empty());
    assert!(defaults.skipped.is_empty());
    let row = sqlx::query("SELECT 'borrowed' AS name")
        .fetch_one(&mut database)
        .await?;
    assert_eq!(Borrowed::from_row(&row)?.name, "borrowed");
    let defaults = sqlx::query_as::<_, GenericDefault<i64>>("SELECT 1 AS present")
        .fetch_one(&mut database)
        .await?;
    assert_eq!(
        defaults,
        GenericDefault {
            present: 1,
            missing: 69
        }
    );
    Ok(())
}

#[tokio::test]
async fn flatten_and_conversion_preserve_values_and_column_decode_errors() -> sqlx::Result<()> {
    let mut database = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
    let row = sqlx::query_as::<_, Nested>("SELECT 17 AS leaf,255 AS byte")
        .fetch_one(&mut database)
        .await?;
    assert_eq!(
        row,
        Nested {
            child: Child { leaf: 17 },
            converted: ConvertedChild(Child { leaf: 17 }),
            byte: Byte(255)
        }
    );
    let error = sqlx::query_as::<_, Nested>("SELECT 17 AS leaf,256 AS byte")
        .fetch_one(&mut database)
        .await
        .unwrap_err();
    assert!(matches!(error, sqlx::Error::ColumnDecode { index, .. } if index == "byte"));
    // A field default applies to a missing column, never to a decode/type error.
    let error = sqlx::query_as::<_, Named<i64>>(
        "SELECT 7 AS userId,'Ada' AS name,42 AS value,'x' AS type,0 AS absent",
    )
    .fetch_one(&mut database)
    .await
    .unwrap_err();
    assert!(matches!(error, sqlx::Error::ColumnDecode { index, .. } if index == "\"absent\""));
    Ok(())
}

#[tokio::test]
async fn json_nullable_and_conversion_keep_database_null_distinct_from_invalid_data()
-> sqlx::Result<()> {
    let mut database = sqlx::SqliteConnection::connect("sqlite::memory:").await?;
    for nullable in ["NULL", "'null'"] {
        let query = format!("SELECT '[\"one\"]' AS items,{nullable} AS nullable,'7' AS byte");
        let row = sqlx::query_as::<_, JsonValues>(sqlx::AssertSqlSafe(query))
            .fetch_one(&mut database)
            .await?;
        assert_eq!(
            row,
            JsonValues {
                items: vec!["one".into()],
                nullable: None,
                byte: Byte(7)
            }
        );
    }
    let row = sqlx::query_as::<_, JsonValues>(
        "SELECT '[\"two\"]' AS items,'[1,2]' AS nullable,'8' AS byte",
    )
    .fetch_one(&mut database)
    .await?;
    assert_eq!(row.nullable, Some(vec![1, 2]));
    let error =
        sqlx::query_as::<_, JsonValues>("SELECT 'not json' AS items,NULL AS nullable,'7' AS byte")
            .fetch_one(&mut database)
            .await
            .unwrap_err();
    assert!(matches!(error, sqlx::Error::ColumnDecode { index, .. } if index == "\"items\""));
    Ok(())
}
