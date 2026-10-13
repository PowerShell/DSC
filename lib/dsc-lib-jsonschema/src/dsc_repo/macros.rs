#[macro_export]
macro_rules! schema_i18n {
    ($dot_path: literal) => {
        Self::schema_i18n($dot_path).unwrap()
    };
}
