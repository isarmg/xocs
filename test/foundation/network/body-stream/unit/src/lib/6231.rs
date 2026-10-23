#[cfg(test)]
fn _assert_bounds() {
    fn can_be_trait_object(_: &dyn Body<Data = std::io::Cursor<Vec<u8>>, Error = std::io::Error>) {}
}
