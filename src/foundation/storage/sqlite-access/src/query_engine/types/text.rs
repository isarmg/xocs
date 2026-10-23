use std::ops::{Deref, DerefMut};

/// Store a Rust value as SQLite `TEXT` using [`Display`] and [`FromStr`].
///
/// This adapter is useful for types with a stable string representation and no
/// direct SQLite conversion. Decoding parses the text and reports parse failures.
/// Use an infallible `Display` implementation, as formatting failure may panic.
///
/// ```
/// use blog_storage_sqlite_access::types::Text;
/// use std::net::SocketAddr;
///
/// # async fn example(pool: &blog_storage_sqlite_access::SqlitePool) -> blog_storage_sqlite_access::Result<()> {
/// let address: SocketAddr = "127.0.0.1:8081".parse().unwrap();
/// let (stored,): (Text<SocketAddr>,) = blog_storage_sqlite_access::query_as("SELECT ?1")
///     .bind(Text(address))
///     .fetch_one(pool)
///     .await?;
/// assert_eq!(stored.0, address);
/// # Ok(())
/// # }
/// ```
///
/// [`Display`]: std::fmt::Display
/// [`FromStr`]: std::str::FromStr
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Text<T>(pub T);

impl<T> Text<T> {
    /// Extract the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Text<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Text<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/* We shouldn't use blanket impls so individual drivers can provide specialized ones.
impl<T, DB> Type<DB> for Text<T>
where
    String: Type<DB>,
    DB: Database,
{
    fn type_info() -> DB::TypeInfo {
        String::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        String::compatible(ty)
    }
}

impl<'q, T, DB> Encode<'q, DB> for Text<T>
where
    T: Display,
    String: Encode<'q, DB>,
    DB: Database,
{
    fn encode_by_ref(&self, buf: &mut <DB as Database>::ArgumentBuffer) -> Result<IsNull, BoxDynError> {
        self.0.to_string().encode(buf)
    }
}

impl<'r, T, DB> Decode<'r, DB> for Text<T>
where
    T: FromStr,
    BoxDynError: From<<T as FromStr>::Err>,
    &'r str: Decode<'r, DB>,
    DB: Database,
{
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        Ok(Text(<&'r str as Decode<'r, DB>>::decode(value)?.parse()?))
    }
}
*/
