#[cfg(test)]
    #[derive(Debug, PartialEq)]
    pub enum Token<'v> {
        None,
        Bool(bool),
        Char(char),
        Str(&'v str),
        F64(f64),
        I64(i64),
        U64(u64),
    }
