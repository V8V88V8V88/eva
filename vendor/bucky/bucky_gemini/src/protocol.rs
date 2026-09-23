//! A module for parsing the protocol of Gemini itself. This includes its requests and responses.
use common::ResponseParseError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// The status code of the gemini response header.
///
/// A gemini response header contains a status code part, and this part is listed as two decimal
/// digits, where the first digit contains the main status code, and the second digit is a
/// specification on top of that code.
pub enum StatusCode {
    /// # 1x INPUT.
    /// This code is returned when a user input is required. It is expected that you will
    /// try to load the same page with a query part added to the request which is user input.
    /// ## META
    /// The META will contain a prompt for the user
    ///
    /// ## Subcodes
    /// - 11: SENSITIVE INPUT. The client should treat it the same as INPUT, but should obfuscate
    /// the input to the user. Used for things like passwords.
    Input(u8),
    /// # 2x SUCCESS.
    /// This code is returned when the page was successfully loaded.
    /// ## META
    /// The META will contain a MIME type for the data sent
    Success(u8),
    /// # 3x REDIRECT.
    /// This code is returned when the server is redirecting the client to a new page.
    /// ## META
    /// The META of the header will contain the page to redirect to
    ///
    /// ## Subcodes
    /// - 30: TEMPORARY REDIRECT.
    /// - 31: PERMANENT REDIRECT. This page will never exist again and is permanently relocated to
    /// the link sent
    Redirect(u8),
    /// # 4x TEMPORARY FAILURE.
    /// This code is returned when there is a failure handling the request that may work later on.
    /// ## META
    /// The META of this header contains additional info about the failure. This should be
    /// displayed to human users.
    ///
    /// ## Additional info:
    /// Aggregators or crawlers should NOT repeat this request
    ///
    /// ## Subcodes
    /// - 41: SERVER UNAVAILABLE. The server is unavailable due to maintainence or slow down.
    /// - 42: CGI ERROR. A CGI process, or similar dynamic content system, has died or timed out
    /// unexpectedly.
    /// - 43: PROXY ERROR. A proxy request failed because the server was unable to successfully
    /// complete a transaction with the remote host.
    /// - 44: SLOW DOWN. Rate limiting is in effect. The META is an integer showing how long the
    /// client should wait before another request is made.
    TemporaryFailure(u8),
    /// # 5x PERMANENT FAILURE
    /// This code is returned when there is a failure handling the request. This request will NEVER
    /// work in the future and will fail in the same way again with an identical request.
    /// ## META
    /// The META of this header contains additional info about the failure and should be shown to
    /// human users.
    ///
    /// ## Additional info:
    /// Aggregators or crawlers should NOT repeat this request
    ///
    /// ## Subcodes:
    /// - 51: NOT FOUND. Akin to HTTP's 404, this request is accesing a resource that is not
    /// available. This resource may be available later on but not in the near future.
    /// - 52: GONE. This resource is gone and will never be in this location again. Search engines
    /// and aggregators should remove this entry and convey to users that this resource is gone.
    /// - 53: PROXY REQUEST REFUSED. This request was made for a different domain and this server
    /// does not except proxy requests.
    /// - 59: BAD REQUEST. The request header was malformed in some way or form.
    PermanentFailure(u8),
    /// # 6x CLIENT CERTIFICATE REQUIRED
    /// This code is returned when the requested resource requires a client certificate. If the
    /// request was made without a client certificate, it should provide one. If it was made with
    /// one, the server did not accept it and should be made with a different certificate.
    /// ## META
    /// The META of the header will contain more information as to why the certificate is required
    /// or as to why the certificate was denied and should be shown to the user
    ///
    /// ## Subcodes:
    /// - 61: CERTIFICATE NOT AUTHORISED. The certificate is not authorised to access the given
    /// resource. The certificate is not necessairly the problem, it is just simply not authorized
    /// to access this specific resource.
    /// - 62: CERTIFICATE NOT VALID. The certificate is not a valid certificate and was not
    /// accepted. Unlike code 61, the certificate itself is the problem. It could be that the
    /// certificate is expired or the start date is in the future, or it could be because it is a
    /// violation of the X509 standard.
    ClientCertRequired(u8),
    /// # Unknown status code
    /// This is returned when the status code returned from the server is unknown.
    /// Unlike the rest of the status codes, contained in this enum variant is the return code in
    /// its entirety. Eg. if the status code is 84, 84 will be contained in the variant and not
    /// just 4.
    Unknown(u8),
}

impl From<u8> for StatusCode {
    fn from(i: u8) -> Self {
        if i > 99 {
            return Self::Unknown(i);
        }
        let first_digit = i % 10;
        let second_digit = i / 10;
        match second_digit {
            1 => Self::Input(first_digit),
            2 => Self::Success(first_digit),
            3 => Self::Redirect(first_digit),
            4 => Self::TemporaryFailure(first_digit),
            5 => Self::PermanentFailure(first_digit),
            6 => Self::ClientCertRequired(first_digit),
            _ => Self::Unknown(i),
        }
    }
}

impl From<StatusCode> for u8 {
    fn from(s: StatusCode) -> Self {
        match s {
            StatusCode::Input(i) => 10 + i,
            StatusCode::Success(i) => 20 + i,
            StatusCode::Redirect(i) => 30 + i,
            StatusCode::TemporaryFailure(i) => 40 + i,
            StatusCode::PermanentFailure(i) => 50 + i,
            StatusCode::ClientCertRequired(i) => 60 + i,
            StatusCode::Unknown(i) => i,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
/// A Gemini response.
///
/// A Gemini response consists of two parts: The header and the content. The header is separated by
/// a new line (CRLF or just LF) and contains two parts in itself, the status code, and a META
/// string with more info about the status code.
///
/// # Creating a Response Struct
/// There are a few ways you can construct a Response struct. You can create it from its raw parts
/// (since all its fields are public), or you can create it using the `TryFrom` implementation (see
/// the [`TryFrom`](#method.try_from) implementation).
pub struct Response {
    /// The status code of the response header.
    pub status: StatusCode,
    /// The META string of the response header.
    pub meta: String,
    /// The data returned from the header.
    pub data: Vec<u8>,
}

impl core::convert::TryFrom<&[u8]> for Response {
    type Error = ResponseParseError;
    /// Parses a response from a u8 slice.
    ///
    /// # Arguments:
    ///
    /// * `raw_response` - The raw response bytes to parse
    ///
    /// # Returns:
    ///
    /// * A Result with either a fully parsed response or an [error describing what went wrong when
    /// parsing](ResponseParseError)
    ///
    /// # Example:
    /// ```
    /// # use bucky_gemini::protocol::Response;
    /// # use bucky_gemini::protocol::StatusCode;
    /// # use std::convert::TryFrom;
    /// # fn main() -> Result<(), common::ResponseParseError> {
    /// let raw_response = r#"20 text/gemini
    /// ## Test response
    /// Hello!"#;
    /// let res = Response::try_from(raw_response.as_bytes()).unwrap();
    /// assert_eq!(res.status, StatusCode::Success(0));
    /// assert_eq!(res.meta, "text/gemini");
    /// assert_eq!(String::from_utf8_lossy(&res.data).into_owned(), "# Test response\nHello!");
    /// # Ok(())
    /// }
    /// ```
    fn try_from(raw_response: &[u8]) -> Result<Self, ResponseParseError> {
        if raw_response.is_empty() {
            return Err(ResponseParseError::EmptyResponse);
        }
        // Let's find the first LF in the response.
        // Since CR is before the LF we can just clip that off if the response contains it
        let mut first_lf = 0;
        for (i, b) in raw_response.iter().enumerate() {
            if *b == b'\n' {
                first_lf = i;
                break;
            }
        }
        // If the first_lf was not found then we can assume that the response header is invalid,
        // since it needs to end in a CRLF
        if first_lf == 0 {
            return Err(ResponseParseError::InvalidResponseHeader);
        }

        // Now we'll convert the slice into a string with the last of the lf
        let Ok(response_header) = core::str::from_utf8(&raw_response[..first_lf]) else {
            return Err(ResponseParseError::InvalidResponseHeader);
        };

        // We'll split on whitespace
        let Some((status_code, meta)) = response_header.split_once(' ') else {
            return Err(ResponseParseError::InvalidResponseHeader);
        };
        // Then we'll trim the meta
        let meta = meta.trim();
        // And then we'll check how long the meta is
        if meta.len() > 1024 {
            return Err(ResponseParseError::InvalidResponseHeader);
        }
        let Ok(status_code) = status_code.parse::<u8>() else {
            return Err(ResponseParseError::InvalidResponseHeader);
        };

        let status = StatusCode::from(status_code);

        let data = Vec::from(&raw_response[first_lf + 1..]);

        Ok(Self {
            status,
            meta: String::from(meta),
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    #[test]
    fn status_code_from_u8_input() {
        assert_eq!(StatusCode::from(18), StatusCode::Input(8));
    }
    #[test]
    fn status_code_to_u8() {
        assert_eq!(u8::from(StatusCode::Input(8)), 18);
    }
    #[test]
    fn response_parse_slice() {
        let raw_response = "20 text/gemini\r\n# Hello!";
        let parsed_response = Response::try_from(raw_response.as_bytes()).unwrap();
        assert_eq!(parsed_response.status, StatusCode::Success(0));
        assert_eq!(parsed_response.meta, "text/gemini");
        assert_eq!(parsed_response.data, "# Hello!".as_bytes());
    }
    #[test]
    fn response_parse_slice_error_empty() {
        let raw_response = "";
        let parsed_response = Response::try_from(raw_response.as_bytes()).unwrap_err();
        assert_eq!(parsed_response, ResponseParseError::EmptyResponse);
    }
    #[test]
    fn response_parse_slice_error_invalid_header_missing_space() {
        let raw_response = "20text/gemini\r\n#Hello!";
        let parsed_response = Response::try_from(raw_response.as_bytes()).unwrap_err();
        assert_eq!(parsed_response, ResponseParseError::InvalidResponseHeader);
    }
    #[test]
    fn response_parse_slice_error_invalid_header_missing_space_and_meta() {
        let raw_response = "20\r\n# Hello!";
        let parsed_response = Response::try_from(raw_response.as_bytes()).unwrap_err();
        assert_eq!(parsed_response, ResponseParseError::InvalidResponseHeader);
    }
    #[test]
    fn response_parse_slice_error_invalid_header_meta_long() {
        let mut raw_response: String = String::from("20 ");
        for _ in 0..2048 {
            raw_response.push('a');
        }
        raw_response.push_str("\r\n# Hello!");
        let parsed_response = Response::try_from(raw_response.as_bytes()).unwrap_err();
        assert_eq!(parsed_response, ResponseParseError::InvalidResponseHeader);
    }
    #[test]
    fn response_parse_slice_empty_body() {
        let raw_response = "20 text/gemini\r\n";
        let parsed_response = Response::try_from(raw_response.as_bytes()).unwrap();
        assert_eq!(parsed_response.status, StatusCode::Success(0));
        assert_eq!(parsed_response.meta, "text/gemini");
        assert_eq!(parsed_response.data, []);
    }
    #[test]
    fn response_parse_slice_empty_meta() {
        let raw_response = "20 \r\n";
        let parsed_response = Response::try_from(raw_response.as_bytes()).unwrap();
        assert_eq!(parsed_response.status, StatusCode::Success(0));
        assert_eq!(parsed_response.meta, "");
        assert_eq!(parsed_response.data, []);
    }
}
