use std::{collections::HashMap, fmt::Write};
use anyhow::{Context, Result};

use crate::status::STATUS_LOOKUP_TABLE;


#[derive(Debug)]
pub(crate) struct Response {
    /// HTTP/1.1 status code
    status_code: u16,
    /// HTTP/1.1 Headers
    headers: HashMap<&'static str, String>,
    /// Response body
    body: String,
}

impl Response {
    pub (crate) fn new(
        status_code: u16,
        headers: HashMap<&'static str, String>,
        body: String
    ) -> Self {
        Self {
            status_code: status_code,
            headers: headers,
            body: body
        }
    }

    pub(crate) fn construct_response(&self) -> Result<String> {
        let mut response = String::with_capacity(96 + self.body.len());

        // Build status line
        let status_reason = STATUS_LOOKUP_TABLE
            .get(&self.status_code)
            .copied()
            .with_context(|| "Status code not found on status table")?;

        write!(response, "HTTP/1.1 {} {status_reason}\r\n", self.status_code)?;

        // Build headers string
        for (key, value) in &self.headers {
            write!(response, "{key}: {value}\r\n")?;
        }
        response.push_str("\r\n"); // end of headers

        // Build body
        if self.headers.contains_key("Content-Length") {
            response.push_str(&self.body);
        }

        Ok(response)
    }

    pub fn build_headers (
        status_code: u16,
        response_body: &str,
        is_octet: bool
    ) -> HashMap<&'static str, String> {
        let mut headers: HashMap<&'static str, String> = HashMap::new();

        // Build headers except for 100, 204, 304 status codes
        // see: https://www.rfc-editor.org/rfc/rfc9110.html#name-content-length
        if !matches!(status_code, 100u16 | 204u16 | 304u16) {
            if !response_body.is_empty() & !is_octet {
                // default content-type to text/plain
                headers.insert("Content-Type", "text/plain".to_string());
            } else if !response_body.is_empty() & is_octet {
                headers.insert("Content-Type", "application/octet-stream".to_string());
            }
            // keep sending content-length even if it is 0
            headers.insert("Content-Length", response_body.len().to_string());
        }

        headers
    }
}


#[cfg(test)]
mod tests {
    use super::Response;
    use std::collections::HashMap;

    #[test]
    #[should_panic(expected = "Status code not found on status table")]
    /// Test for unavailable status code
    fn unavailable_status_code_test() {
        Response::new(9999, HashMap::new(), String::new())
            .construct_response()
            .unwrap();
    }

    #[test]
    fn headers_builder_test() {
        let headers1 = Response::build_headers(200, "abc", false);
        assert_eq!(
            headers1,
            HashMap::from([
                ("Content-Length", "3".to_string()),
                ("Content-Type", "text/plain".to_string())
            ])
        );

        let headers2 = Response::build_headers(200, "abc", true);
        assert_eq!(
            headers2,
            HashMap::from([
                ("Content-Length", "3".to_string()),
                ("Content-Type", "application/octet-stream".to_string())
            ])
        );

         let headers3 = Response::build_headers(200, "", true);
        assert_eq!(
            headers3,
            HashMap::from([
                ("Content-Length", "0".to_string()),
            ])
        );

        // build headers except for 100, 204, and 304
        let headers4 = Response::build_headers(100, "abc", false);
        let headers5 = Response::build_headers(204, "abc", true);
        let headers6 = Response::build_headers(304, "abc", false);
        assert_eq!(headers4, HashMap::new());
        assert_eq!(headers5, HashMap::new());
        assert_eq!(headers6, HashMap::new());
    }

    #[test]
    /// Test response builder with status-line, headers, and body
    fn construct_response_test() {
        let status_code = 200;
        let body = String::from("abc");
        let headers = Response::build_headers(status_code, &body, false);
    
        let response_str = Response::new(status_code, headers, body).construct_response().unwrap();

        assert!(response_str.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(response_str.contains("Content-Type: text/plain\r\n"));
        assert!(response_str.contains("Content-Length: 3\r\n"));
        assert!(response_str.ends_with("\r\nabc"));
    }

    #[test]
    /// Test response builder for 100 or 204 or 304 status codes
    /// which means there is no body, no content-length and no content-type
    fn construct_response_with_no_content_length_test() {
        let status_code = 204;
        let body = String::from("abc");
        let headers = Response::build_headers(status_code, &body, false);
    
        let response_str = Response::new(status_code, headers, body).construct_response().unwrap();

        assert!(response_str.starts_with("HTTP/1.1 204 No Content\r\n"));
    }
}