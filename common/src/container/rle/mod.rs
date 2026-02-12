pub mod bits;
pub mod png;

#[derive(Debug, Clone, Copy)]
pub struct Run<T = u8> {
    pub length: u64,
    pub value: T,
}
