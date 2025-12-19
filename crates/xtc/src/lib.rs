use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum XtcError {
    #[error("XTC functionality not implemented yet")]
    NotImplemented,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct XtcMetadata {
    pub title: String,
}

impl XtcMetadata {
    pub fn new<T: Into<String>>(title: T) -> Self {
        Self {
            title: title.into(),
        }
    }
}

pub fn placeholder_encode(_metadata: &XtcMetadata) -> Result<(), XtcError> {
    tracing::trace!("placeholder encode called");
    Err(XtcError::NotImplemented)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_encode_is_unimplemented() {
        let metadata = XtcMetadata::new("Example");
        let result = placeholder_encode(&metadata);
        assert!(matches!(result, Err(XtcError::NotImplemented)));
    }
}
