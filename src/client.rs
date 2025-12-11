// Client module - kept for potential future use as a client library
// Currently unused as this is a server implementation

#[allow(dead_code)]
pub struct GummySearchClient {
    base_url: String,
}

#[allow(dead_code)]
impl GummySearchClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}
