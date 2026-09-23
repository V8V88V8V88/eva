#![warn(clippy::all, clippy::pedantic)]

use {
    common::{Content, ToGmi},
    std::{fmt, io, path::PathBuf},
    url::Url,
};

#[derive(Debug)]
pub enum FileUrlError {
    InvalidFileUrl,
    EmptyPath,
    ReadError(io::Error),
    MetadataError(io::Error),
    FileTypeError,
}

impl fmt::Display for FileUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFileUrl => write!(f, "Invalid File Url"),
            Self::EmptyPath => write!(f, "Empty Path"),
            Self::ReadError(e) => write!(f, "Read Error: {e}"),
            Self::MetadataError(e) => write!(f, "Metadata Error: {e}"),
            Self::FileTypeError => write!(f, "File Type Error"),
        }
    }
}

impl std::error::Error for FileUrlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ReadError(e) | Self::MetadataError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for FileUrlError {
    fn from(value: io::Error) -> Self {
        Self::ReadError(value)
    }
}

/// Attempts to open a connection and return the server's response
/// # Errors
/// - multiple error conditions
pub fn try_open(url: &Url) -> Result<Content, FileUrlError> {
    if url.scheme() != "file" {
        return Err(FileUrlError::InvalidFileUrl);
    }
    let mut path = url.host_str().unwrap_or("").to_string();
    path.push_str(url.path());
    if path.is_empty() {
        return Err(FileUrlError::EmptyPath);
    }
    let path = PathBuf::from(path);
    let path = if let Ok(p) = std::fs::canonicalize(&path) {
        p
    } else {
        path
    };
    match std::fs::metadata(&path) {
        Ok(meta) => {
            if meta.is_dir() {
                let gmi = path.to_gmi()?;
                Ok(Content {
                    url: None,
                    mime: String::from("text/gemini"),
                    bytes: gmi.as_bytes().to_vec(),
                })
            } else if meta.is_file() {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        let mut mime = tree_magic_mini::from_u8(&bytes).to_string();
                        if mime.starts_with("text/") {
                            mime = match &path.extension().map(std::ffi::OsStr::to_str) {
                                Some(Some("gmi" | "gemini")) => String::from("text/gemini"),
                                _ => mime,
                            }
                        }
                        Ok(Content {
                            url: None,
                            mime,
                            bytes,
                        })
                    }
                    Err(e) => Err(FileUrlError::ReadError(e)),
                }
            } else {
                Err(FileUrlError::FileTypeError)
            }
        }
        Err(e) => Err(FileUrlError::MetadataError(e)),
    }
}
