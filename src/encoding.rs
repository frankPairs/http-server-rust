#[derive(Debug)]
pub enum CompressionSchema {
    Gzip,
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
