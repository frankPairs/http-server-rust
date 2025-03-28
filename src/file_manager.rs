use anyhow::Context;
use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::PathBuf,
};

#[derive(Debug)]
pub enum FileManagerError {
    NotFound,
    Uknown(anyhow::Error),
}

impl std::fmt::Display for FileManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileManagerError::NotFound => {
                write!(f, "File not found")
            }
            FileManagerError::Uknown(err) => {
                write!(f, "{}\n\t{}", err, err.root_cause())
            }
        }
    }
}

pub struct ReadResult {
    pub content: String,
    pub bytes_read: usize,
}

#[derive(Debug)]
pub struct FileManager;

impl FileManager {
    pub fn read(path: PathBuf) -> Result<ReadResult, FileManagerError> {
        let exists = path
            .try_exists()
            .context("Path exists")
            .map_err(FileManagerError::Uknown)?;

        if !exists {
            return Err(FileManagerError::NotFound);
        }

        let file = File::open(&path)
            .context(format!("Open file from path {:?}", path))
            .map_err(FileManagerError::Uknown)?;

        let mut reader = BufReader::new(file);
        let mut content = String::new();

        let bytes_read = reader
            .read_to_string(&mut content)
            .context(format!("Read file from path {:?}", path))
            .map_err(FileManagerError::Uknown)?;

        Ok(ReadResult {
            content,
            bytes_read,
        })
    }

    pub fn write(directory: &str, filename: &str, content: &str) -> Result<(), FileManagerError> {
        fs::create_dir_all(directory)
            .with_context(|| format!("Create {:?} directory", directory))
            .map_err(FileManagerError::Uknown)?;

        let mut file_path = PathBuf::new();

        file_path.push(directory);
        file_path.push(filename);

        let mut file = File::create(&file_path)
            .with_context(|| format!("Create {:?} file", file_path))
            .map_err(FileManagerError::Uknown)?;

        file.write_all(content.as_bytes())
            .with_context(|| format!("Write {:?} file", filename))
            .map_err(FileManagerError::Uknown)?;

        Ok(())
    }
}
