use {
    sha2::{Digest, Sha256},
    std::{
        collections::BTreeMap,
        fmt::Write,
        fs, io,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Certificate {
    pub fingerprint: String,
    pub expires: i64,
}

impl Certificate {
    /// # Errors
    /// Returns an error if the certificate cannot be parsed
    pub fn from_der(der: &[u8]) -> Result<Self, String> {
        let (_, cert) = x509_parser::parse_x509_certificate(der).map_err(|e| e.to_string())?;
        Ok(Self {
            fingerprint: fingerprint(cert.public_key().raw),
            expires: cert.validity().not_after.timestamp(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Trust {
    New,
    Known,
    Renewed,
    Changed,
}

#[derive(Debug)]
pub struct KnownHosts {
    path: PathBuf,
    hosts: BTreeMap<String, Certificate>,
}

impl KnownHosts {
    #[must_use]
    pub fn load(path: &Path) -> Self {
        let hosts = fs::read_to_string(path)
            .map(|s| s.lines().filter_map(parse_line).collect())
            .unwrap_or_default();
        Self {
            path: path.to_path_buf(),
            hosts,
        }
    }

    #[must_use]
    pub fn check(&self, host: &str, cert: &Certificate) -> Trust {
        self.check_at(host, cert, now())
    }

    fn check_at(&self, host: &str, cert: &Certificate, now: i64) -> Trust {
        match self.hosts.get(host) {
            None => Trust::New,
            Some(known) if known == cert => Trust::Known,
            Some(known) if known.fingerprint == cert.fingerprint || known.expires < now => {
                Trust::Renewed
            }
            Some(_) => Trust::Changed,
        }
    }

    /// # Errors
    /// Returns an error if the known hosts file cannot be written
    pub fn trust(&mut self, host: &str, cert: &Certificate) -> io::Result<()> {
        self.hosts.insert(host.to_string(), cert.clone());
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir)?;
        }
        let mut out = String::new();
        for (host, cert) in &self.hosts {
            let _ = writeln!(out, "{host} {} {}", cert.fingerprint, cert.expires);
        }
        fs::write(&self.path, out)
    }
}

fn parse_line(line: &str) -> Option<(String, Certificate)> {
    let mut fields = line.split_whitespace();
    let host = fields.next()?.to_string();
    let fingerprint = fields.next()?.to_string();
    let expires = fields
        .next()
        .and_then(|e| e.parse().ok())
        .unwrap_or(i64::MAX);
    Some((
        host,
        Certificate {
            fingerprint,
            expires,
        },
    ))
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[must_use]
pub fn fingerprint(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cert(fingerprint: &str, expires: i64) -> Certificate {
        Certificate {
            fingerprint: fingerprint.to_string(),
            expires,
        }
    }

    #[test]
    fn trust_roundtrip() {
        let dir = std::env::temp_dir().join(format!("bucky-tofu-{}", std::process::id()));
        let path = dir.join("known_hosts");
        let host = "example.org:1965";
        let mut known = KnownHosts::load(&path);
        assert_eq!(known.check_at(host, &cert("aa", 200), 100), Trust::New);
        known.trust(host, &cert("aa", 200)).unwrap();
        let known = KnownHosts::load(&path);
        assert_eq!(known.check_at(host, &cert("aa", 200), 100), Trust::Known);
        assert_eq!(known.check_at(host, &cert("aa", 300), 100), Trust::Renewed);
        assert_eq!(known.check_at(host, &cert("bb", 300), 100), Trust::Changed);
        assert_eq!(known.check_at(host, &cert("bb", 300), 250), Trust::Renewed);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn legacy_line_never_expires() {
        let (_, c) = parse_line("example.org:1965 aa").unwrap();
        assert_eq!(c.expires, i64::MAX);
    }

    #[test]
    fn parses_real_certificate() {
        let rcgen::CertifiedKey {
            cert: generated, ..
        } = rcgen::generate_simple_self_signed(vec!["example.org".to_string()]).unwrap();
        let c = Certificate::from_der(generated.der()).unwrap();
        assert_eq!(c.fingerprint.len(), 64);
        assert!(c.expires > now());
    }
}
