//! A library for making requests to gemini servers and parsing
//! reponses. This library requires acccess to `std::net` and
//! `openssl`.
//!

use super::protocol;
use common::RequestError;
use native_tls::TlsConnector;
use url::Url;

use std::convert::TryFrom;
use std::time::Duration;

/// Contains a request to a server
///
/// # Creating the struct
/// You can create this struct by either creating it from its raw parts, or using
/// any From implementaiton for this struct.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Request {
    /// The raw [String] of the request. You can use this
    /// when actually making requests using whatever backend you
    /// may like
    pub raw_string: String,
}

impl From<&Url> for Request {
    /// Create a [`Request`] from a [`Url`](url::Url) struct. You can also use [`Request::from()`]
    ///
    /// # Example
    /// ```
    /// # use url::Url;
    /// # use bucky_gemini::request::Request;
    /// # fn main() -> Result<(), url::ParseError> {
    /// use std::convert::TryFrom;
    /// let url = Url::parse("gemini://gemini.circumlunar.space")?;
    /// let req = Request::from(&url);
    /// assert_eq!(req.raw_string, "gemini://gemini.circumlunar.space\r\n");
    /// # Ok(())
    /// # }
    fn from(url: &Url) -> Self {
        let mut raw_string = url.to_string();
        raw_string.push_str("\r\n");
        Self { raw_string }
    }
}

impl core::fmt::Display for Request {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.raw_string)
    }
}

// This is for the TLS for Gemini. This will just simply trust any TLS
// certs we get for now. We can implement TOFU later on.
//struct DummyVerifier {}

/*impl DummyVerifier {
    pub fn new() -> Self {
        Self {}
    }
}*/

/*impl rustls::client::ServerCertVerifier for DummyVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}*/

/*
struct TofuVerifier {
    pub certs: std::collections::HashMap<rustls::ServerName, rustls::Certificate>,
}

impl TofuVerifier {
    pub fn new() -> Self {
        Self {
            certs: std::collections::HashMap::new(),
        }
    }
}

impl rustls::client::ServerCertVerifier for TofuVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
*/

/// Open a TCP stream to a [`Url`](gmi::url::Url) given with a default port listed.
fn open_tcp_stream(url: &Url, default_port: u16) -> Result<std::net::TcpStream, RequestError> {
    let Some(host) = url.host_str() else {
        return Err(RequestError::DnsError);
    };
    let port = url.port().unwrap_or(default_port);
    let tcp_stream = common::connect(format!("{host}:{port}"), Duration::new(10, 0))
        .map_err(RequestError::IoError)?;
    Ok(tcp_stream)
}

/// Use a stream given `std::io::Write` to write a request
fn use_stream_do_request(req: &str, stream: &mut dyn std::io::Write) -> Result<(), RequestError> {
    _ = stream.write(req.as_bytes())?;
    Ok(())
}

/// Use a stream `std::io::Read` to read a response and parse that response
fn use_stream_get_resp(stream: &mut dyn std::io::Read) -> Result<protocol::Response, RequestError> {
    let mut buffer: Vec<u8> = Vec::new();
    stream.read_to_end(&mut buffer)?;
    parse_merc_gemini_resp(&buffer)
}

/// Parse a response taken from a server
fn parse_merc_gemini_resp(resp: &[u8]) -> Result<protocol::Response, RequestError> {
    match protocol::Response::try_from(resp) {
        Ok(r) => Ok(r),
        Err(e) => Err(RequestError::ResponseParseError(e)),
    }
}

/// Make a request to a gemini server
fn make_gemini_request(url: &Url) -> Result<protocol::Response, RequestError> {
    // These are only needed in this funcion, so we'll put a use here.
    //use rustls::client::{ClientConfig, ClientConnection};
    //use std::sync::Arc;

    // Get our request string
    let request = Request::from(url);

    //let authority = match url.host_str() {
    //    Some(h) => h.to_string(),
    //    None => return Err(RequestError::DnsError),
    //};
    let port = url.port().unwrap_or(1965);

    // Get our DNS name
    //let dnsname = match authority.as_str().try_into() {
    //    Ok(s) => s,
    //    Err(_) => return Err(RequestError::DnsError),
    //};

    // Set up rustls
    //let cfg = ClientConfig::builder()
    //    .with_safe_defaults()
    //    .with_custom_certificate_verifier(Arc::new(DummyVerifier::new()))
    //    .with_no_client_auth();

    // Set up our TLS client
    //let client = ClientConnection::new(Arc::new(cfg), dnsname).unwrap();

    let connector = TlsConnector::builder()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    // Open up a socket
    let tcp_stream = open_tcp_stream(url, port)?;
    //let mut tls_stream = rustls::StreamOwned::new(client, tcp_stream);
    let host = url.host_str().unwrap_or("");
    let tls_stream = connector.connect(host, tcp_stream);
    let mut tls_stream = match tls_stream {
        Err(e) => return Err(RequestError::TlsError(format!("{e:?}"))),
        Ok(stream) => stream,
    };

    use_stream_do_request(request.raw_string.as_str(), &mut tls_stream)?;
    use_stream_get_resp(&mut tls_stream)
}

/// Make a request to a mercury server
fn make_mercury_request(url: &Url) -> Result<protocol::Response, RequestError> {
    let request = Request::from(url);
    let mut stream = open_tcp_stream(url, 1963)?;
    use_stream_do_request(request.raw_string.as_str(), &mut stream)?;
    use_stream_get_resp(&mut stream)
}

/// Make a request to a [URL](url::Url). The scheme will default to gemini
///
/// # Errors
/// Will return a [`RequestError`] on any sort of error
/// # Example:
/// ```no_run
/// # use bucky_gemini::request;
/// # use bucky_gemini::protocol::StatusCode;
/// # use url::Url;
/// # fn main() -> Result<(), common::RequestError> {
/// use std::convert::TryFrom;
/// let url = Url::parse("gemini://gemini.circumlunar.space/").unwrap();
/// let response = request::request(&url)?;
/// assert_eq!(response.status, StatusCode::Success(0));
/// # Ok(())
/// # }
/// ```
pub fn request(url: &Url) -> Result<protocol::Response, RequestError> {
    // Get the scheme, and see what type of request we're making
    match url.scheme() {
        "gemini" => make_gemini_request(url),
        "mercury" => make_mercury_request(url),
        s => Err(RequestError::UnknownScheme(String::from(s))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    #[test]
    fn make_request_invalid_scheme_error() {
        let raw_url = Url::try_from("https://example.com").unwrap();
        let response = request(&raw_url).unwrap_err();
        match response {
            RequestError::UnknownScheme(s) => {
                assert_eq!(s, "https");
            }
            e => {
                panic!(
                    "Error returned was not an UnknownScheme but instead {:?}",
                    e
                );
            }
        }
    }
}
