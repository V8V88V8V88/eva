use std::{error, fmt, fs, io, path::PathBuf};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Rcgen(rcgen::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Rcgen(e) => write!(f, "{e}"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rcgen::Error> for Error {
    fn from(e: rcgen::Error) -> Self {
        Self::Rcgen(e)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Rcgen(e) => Some(e),
        }
    }
}

/// Generates a client certificate for the specified host
/// # Errors
/// - problem generating a certificate
/// - problem serializing the generated certificate
/// - problem writing the certificate to disk
pub fn generate(host: &str, certdir: PathBuf) -> Result<(), Error> {
    let mut f = certdir;
    f.push(&format!("{host}.pem"));
    let alt_names = vec![host.to_string(), "localhost".to_string()];
    let rcgen::CertifiedKey { cert, .. } = rcgen::generate_simple_self_signed(alt_names)?;
    let pem = cert.pem();
    fs::write(f, pem)?;
    Ok(())
}
