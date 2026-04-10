#[macro_export]
macro_rules! schema_i18n {
    ($dotPath: literal) => {
        Self::schema_i18n($dotPath).unwrap()
    };
}
