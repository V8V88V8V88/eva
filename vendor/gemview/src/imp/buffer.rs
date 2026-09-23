#[derive(Clone, Debug, Default)]
pub struct Buffer {
    pub mime: String,
    pub content: Vec<u8>,
}
