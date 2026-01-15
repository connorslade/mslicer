#[macro_export]
macro_rules! include_asset {
    ($name:expr) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $name))
    };
}
