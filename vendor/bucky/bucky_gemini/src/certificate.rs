use {
    sha2::{Digest, Sha256},
    std::{
        error, fmt, fs, io,
        path::{Path, PathBuf},
    },
    url::Url,
};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Rcgen(rcgen::Error),
    Tls(native_tls::Error),
    InvalidPem,
    InvalidUrl,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{e}"),
            Self::Rcgen(e) => write!(f, "{e}"),
            Self::Tls(e) => write!(f, "{e}"),
            Self::InvalidPem => write!(f, "Identity file is missing a certificate or key"),
            Self::InvalidUrl => write!(f, "Url has no host"),
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

impl From<native_tls::Error> for Error {
    fn from(e: native_tls::Error) -> Self {
        Self::Tls(e)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Rcgen(e) => Some(e),
            Self::Tls(e) => Some(e),
            Self::InvalidPem | Self::InvalidUrl => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scope {
    pub host: String,
    pub port: u16,
    pub path: String,
}

const SCOPE_PREFIX: &str = "# scope ";

impl Scope {
    #[must_use]
    pub fn from_url(url: &Url) -> Option<Self> {
        let path = match url.path() {
            "" => "/",
            p => p,
        };
        Some(Self {
            host: url.host_str()?.to_lowercase(),
            port: url.port().unwrap_or(1965),
            path: path.to_string(),
        })
    }

    #[must_use]
    pub fn site(url: &Url) -> Option<Self> {
        Self::from_url(url).map(|s| Self {
            path: String::from("/"),
            ..s
        })
    }

    #[must_use]
    pub fn matches(&self, url: &Url) -> bool {
        let Some(other) = Self::from_url(url) else {
            return false;
        };
        if other.host != self.host || other.port != self.port {
            return false;
        }
        let path = other.path.as_str();
        let base = self.path.as_str();
        path == base
            || (path.starts_with(base)
                && (base.ends_with('/') || path[base.len()..].starts_with('/')))
    }

    fn file_name(&self) -> String {
        let host: String = self
            .host
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' })
            .collect();
        let digest = Sha256::digest(self.path.as_bytes());
        let tag: String = digest[..6].iter().map(|b| format!("{b:02x}")).collect();
        format!("{host}_{}_{tag}.pem", self.port)
    }

    fn to_line(&self) -> String {
        format!("{SCOPE_PREFIX}{} {} {}\n", self.host, self.port, self.path)
    }

    fn from_line(line: &str) -> Option<Self> {
        let mut fields = line.strip_prefix(SCOPE_PREFIX)?.splitn(3, ' ');
        Some(Self {
            host: fields.next()?.to_string(),
            port: fields.next()?.parse().ok()?,
            path: fields.next()?.trim_end().to_string(),
        })
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.port == 1965 {
            write!(f, "gemini://{}{}", self.host, self.path)
        } else {
            write!(f, "gemini://{}:{}{}", self.host, self.port, self.path)
        }
    }
}

/// # Errors
/// - problem generating a certificate
/// - problem writing the certificate to disk
pub fn generate(scope: &Scope, certdir: &Path) -> Result<PathBuf, Error> {
    let mut params = rcgen::CertificateParams::new(vec![scope.host.clone()])?;
    params
        .distinguished_name
        .push(rcgen::DnType::CommonName, scope.host.as_str());
    let key = rcgen::KeyPair::generate()?;
    let cert = params.self_signed(&key)?;
    let mut pem = scope.to_line();
    pem.push_str(&cert.pem());
    pem.push_str(&key.serialize_pem());
    fs::create_dir_all(certdir)?;
    let path = certdir.join(scope.file_name());
    write_private(&path, pem.as_bytes())?;
    Ok(path)
}

#[cfg(unix)]
fn write_private(path: &Path, data: &[u8]) -> io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    f.write_all(data)
}

#[cfg(not(unix))]
fn write_private(path: &Path, data: &[u8]) -> io::Result<()> {
    fs::write(path, data)
}

#[must_use]
pub fn find(url: &Url, certdir: &Path) -> Option<(Scope, PathBuf)> {
    fs::read_dir(certdir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "pem"))
        .filter_map(|p| {
            let text = fs::read_to_string(&p).ok()?;
            let scope = Scope::from_line(text.lines().next()?)?;
            scope.matches(url).then_some((scope, p))
        })
        .max_by_key(|(scope, _)| scope.path.len())
}

/// # Errors
/// Returns an error if the file exists but cannot be removed
pub fn remove(url: &Url, certdir: &Path) -> io::Result<()> {
    match find(url, certdir) {
        Some((_, path)) => fs::remove_file(path),
        None => Ok(()),
    }
}

/// # Errors
/// Returns an error if the identity file cannot be read or parsed
pub fn load(url: &Url, certdir: &Path) -> Result<Option<native_tls::Identity>, Error> {
    let Some((_, path)) = find(url, certdir) else {
        return Ok(None);
    };
    let pem = fs::read_to_string(&path)?;
    let cert_start = pem
        .find("-----BEGIN CERTIFICATE-----")
        .ok_or(Error::InvalidPem)?;
    let key_start = pem
        .find("-----BEGIN PRIVATE KEY-----")
        .ok_or(Error::InvalidPem)?;
    if key_start < cert_start {
        return Err(Error::InvalidPem);
    }
    let cert = &pem[cert_start..key_start];
    let key = &pem[key_start..];
    Ok(Some(native_tls::Identity::from_pkcs8(
        cert.as_bytes(),
        key.as_bytes(),
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn scope_matches_path_and_below() {
        let scope = Scope::from_url(&url("gemini://example.org/app")).unwrap();
        assert!(scope.matches(&url("gemini://example.org/app")));
        assert!(scope.matches(&url("gemini://example.org/app/")));
        assert!(scope.matches(&url("gemini://example.org/app/garden?x")));
        assert!(!scope.matches(&url("gemini://example.org/apple")));
        assert!(!scope.matches(&url("gemini://example.org/")));
        assert!(!scope.matches(&url("gemini://example.org:1966/app")));
        assert!(!scope.matches(&url("gemini://other.org/app")));
    }

    #[test]
    fn site_scope_matches_everything_on_host_and_port() {
        let scope = Scope::site(&url("gemini://example.org/deep/page")).unwrap();
        assert!(scope.matches(&url("gemini://example.org/")));
        assert!(scope.matches(&url("gemini://example.org/anything")));
        assert!(!scope.matches(&url("gemini://example.org:1966/")));
    }

    #[test]
    fn scope_line_round_trips() {
        let scope = Scope::from_url(&url("gemini://example.org:1966/a b/")).unwrap();
        assert_eq!(Scope::from_line(scope.to_line().trim_end()), Some(scope));
    }

    #[test]
    fn generate_find_load_remove() {
        let dir = std::env::temp_dir().join(format!("bucky-cert-test-{}", std::process::id()));
        let app = url("gemini://example.org/app");
        let deeper = url("gemini://example.org/app/private");
        assert!(find(&app, &dir).is_none());
        generate(&Scope::from_url(&app).unwrap(), &dir).unwrap();
        generate(&Scope::from_url(&deeper).unwrap(), &dir).unwrap();
        assert_eq!(find(&app, &dir).unwrap().0.path, "/app");
        assert_eq!(
            find(&url("gemini://example.org/app/private/x"), &dir).unwrap().0.path,
            "/app/private"
        );
        assert!(find(&url("gemini://example.org/"), &dir).is_none());
        assert!(load(&deeper, &dir).unwrap().is_some());
        remove(&deeper, &dir).unwrap();
        assert_eq!(find(&deeper, &dir).unwrap().0.path, "/app");
        remove(&app, &dir).unwrap();
        assert!(load(&app, &dir).unwrap().is_none());
        _ = fs::remove_dir_all(dir);
    }
}
