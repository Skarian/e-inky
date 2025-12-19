use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncPlan {
    pub target: String,
    pub books: Vec<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SyncError {
    #[error("no books queued for sync")]
    Empty,
}

pub fn build_sync_plan<T: Into<String>>(target: T, books: Vec<String>) -> Result<SyncPlan, SyncError> {
    tracing::trace!("building placeholder sync plan for {} book(s)", books.len());

    if books.is_empty() {
        Err(SyncError::Empty)
    } else {
        Ok(SyncPlan {
            target: target.into(),
            books,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_sync_plan_requires_books() {
        let result = build_sync_plan("/tmp/device", vec![]);
        assert_eq!(result, Err(SyncError::Empty));
    }

    #[test]
    fn build_sync_plan_tracks_targets_and_books() {
        let plan = build_sync_plan("/tmp/device", vec!["book.xtc".to_owned()])
            .expect("plan should be created");
        assert_eq!(plan.target, "/tmp/device");
        assert_eq!(plan.books, vec!["book.xtc".to_owned()]);
    }
}
