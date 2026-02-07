pub mod png;

#[derive(Debug, Clone, Copy)]
pub struct Run {
    pub length: u64,
    pub value: u8,
}
