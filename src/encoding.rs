use anyhow::Context;
use flate2::{write::GzEncoder, Compression};
use std::io::Write;

#[derive(Debug, Clone)]
pub enum CompressionSchema {
    Gzip,
}

impl CompressionSchema {
    pub fn compress(&self, body: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
        match self {
            CompressionSchema::Gzip => GzipCompressor::compress(body),
        }
    }
}

impl std::fmt::Display for CompressionSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionSchema::Gzip => write!(f, "gzip"),
        }
    }
}

impl TryFrom<String> for CompressionSchema {
    type Error = CompressionSchemaError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "gzip" => Ok(CompressionSchema::Gzip),
            _ => Err(CompressionSchemaError::Unknown),
        }
    }
}

#[derive(Debug)]
pub enum CompressionSchemaError {
    Unknown,
}

impl std::fmt::Display for CompressionSchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionSchemaError::Unknown => write!(f, "Unknown compression schema"),
        }
    }
}

struct GzipCompressor;

impl GzipCompressor {
    fn compress(value: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

        encoder
            .write_all(&value)
            .context("Add value to the gzip encoder")?;

        let result = encoder
            .finish()
            .with_context(|| format!("Encoded value = {:?}", value))?;

        Ok(result)
    }
}
