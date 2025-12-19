use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BookMetadata {
    pub identifier: String,
    pub title: String,
}

impl BookMetadata {
    pub fn new<T: Into<String>>(identifier: T, title: T) -> Self {
        Self {
            identifier: identifier.into(),
            title: title.into(),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LibraryError {
    #[error("book not found: {0}")]
    NotFound(String),
}

pub fn find_book(metadata: &[BookMetadata], id: &str) -> Result<BookMetadata, LibraryError> {
    metadata
        .iter()
        .find(|book| book.identifier == id)
        .cloned()
        .ok_or_else(|| LibraryError::NotFound(id.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_book_returns_requested_item() {
        let books = vec![
            BookMetadata::new("id-1", "First"),
            BookMetadata::new("id-2", "Second"),
        ];

        let found = find_book(&books, "id-2").expect("book should be found");
        assert_eq!(found.title, "Second");
    }

    #[test]
    fn find_book_reports_missing_items() {
        let books = vec![BookMetadata::new("id-1", "First")];

        let error = find_book(&books, "unknown").expect_err("book should be missing");
        assert_eq!(error, LibraryError::NotFound("unknown".to_owned()));
    }
}
