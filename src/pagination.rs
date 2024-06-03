use serde::Deserialize;

/// Url paramater for pagination
#[derive(Debug, Deserialize)]
pub struct Pagination {
    /// Index offset in entries
    pub offset: usize,
    /// Maximum number of entries returned
    pub limit: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { offset: 0, limit: usize::MAX }
    }
}
