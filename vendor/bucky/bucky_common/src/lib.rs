#![warn(clippy::all, clippy::pedantic)]

use std::{io, path::PathBuf, string::FromUtf8Error};

#[derive(Clone, Debug)]
pub struct Content {
    pub url: Option<String>,
    pub mime: String,
    pub bytes: Vec<u8>,
}

impl Content {
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            url: None,
            mime: tree_magic_mini::from_u8(&bytes).to_string(),
            bytes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Input {
    pub meta: String,
    pub url: String,
    pub sensitive: u8,
}

#[derive(Clone, Debug)]
pub enum Response {
    Success(Content),
    Redirect(String),
    RequestInput(Input),
    Error(String),
}

#[derive(Debug)]
/// A catch-all enum for any errors that may happen
/// while making and parsing the request
pub enum RequestError {
    /// Occurs when an [IO Error](std::io::Error) occurs.
    IoError(std::io::Error),
    /// Occurs when a DNS error occurs.
    DnsError,
    /// Occurs when some sort of [TLS error](native_tls::Error) occurs
    TlsError(String),
    //TlsError(rustls::Error),
    /// Occurs when the scheme given is unknown. Returns the scheme name.
    UnknownScheme(String),
    /// Occurs when the response from the server cannot be parsed.
    ResponseParseError(ResponseParseError),
    /// Occurs if the request path is not valid
    InvalidPath,
}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::IoError(e) => {
                write!(f, "IO error: {e}")
            }
            Self::DnsError => {
                write!(f, "DNS Error")
            }
            Self::TlsError(e) => {
                write!(f, "TLS Error: {e}")
            }
            Self::UnknownScheme(s) => {
                write!(f, "Unknown scheme {s}")
            }
            Self::ResponseParseError(e) => {
                write!(f, "Response parse error: {e}")
            }
            Self::InvalidPath => {
                write!(f, "Invalid Path")
            }
        }
    }
}

impl std::error::Error for RequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IoError(e) => Some(e),
            Self::ResponseParseError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RequestError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<ResponseParseError> for RequestError {
    fn from(error: ResponseParseError) -> Self {
        Self::ResponseParseError(error)
    }
}

impl From<FromUtf8Error> for RequestError {
    fn from(_error: FromUtf8Error) -> Self {
        Self::InvalidPath
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// An error in parsing a response header from a server
pub enum ResponseParseError {
    /// The entire response was empty.
    EmptyResponse,
    /// The response header was invalid and could not be parsed
    InvalidResponseHeader,
}

impl std::fmt::Display for ResponseParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ResponseParseError::EmptyResponse => {
                write!(f, "Error parsing response! The response was empty!")
            }
            ResponseParseError::InvalidResponseHeader => {
                write!(
                    f,
                    "Error parsing response! The response's header was invalid"
                )
            }
        }
    }
}

impl std::error::Error for ResponseParseError {}

pub trait ToGmi {
    type Error;

    /// Converts the given type to gemtext
    /// # Errors
    /// specified in `Self::Error`
    fn to_gmi(&self) -> Result<String, Self::Error>;
}

impl ToGmi for PathBuf {
    type Error = io::Error;

    fn to_gmi(&self) -> Result<String, Self::Error> {
        let mut page = format!("# Index of {}\n", &self.display());
        if let Some(parent) = self.parent() {
            let link = format!("=> file://{} parent directory\n\n", parent.display(),);
            page.push_str(&link);
        }
        let entries = std::fs::read_dir(self)?;
        for entry in entries.flatten() {
            let link = format!(
                "=> file://{} {}\n",
                entry.path().display(),
                entry.file_name().to_string_lossy(),
            );
            page.push_str(&link);
        }
        Ok(page)
    }
}
