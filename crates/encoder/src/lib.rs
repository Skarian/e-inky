use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum EncoderError {
    #[error("encoder pipeline not ready")]
    NotReady,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncoderConfig {
    pub dither_percent: u8,
}

impl EncoderConfig {
    pub fn new(dither_percent: u8) -> Self {
        Self { dither_percent }
    }
}

pub fn encode_buffer(config: &EncoderConfig, input: &[u8]) -> Result<Vec<u8>, EncoderError> {
    tracing::trace!(
        "encoding placeholder buffer with dither {}% ({} bytes)",
        config.dither_percent,
        input.len()
    );

    if input.is_empty() {
        Err(EncoderError::NotReady)
    } else {
        Ok(input.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_buffer_passes_through_content() {
        let config = EncoderConfig::new(50);
        let input = vec![1_u8, 2, 3];
        let output = encode_buffer(&config, &input).expect("expected placeholder success");
        assert_eq!(output, input);
    }

    #[test]
    fn encode_buffer_signals_not_ready_on_empty_input() {
        let config = EncoderConfig::new(0);
        let result = encode_buffer(&config, &[]);
        assert_eq!(result, Err(EncoderError::NotReady));
    }
}
