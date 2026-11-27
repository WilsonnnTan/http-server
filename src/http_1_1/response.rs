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
        response.push_str(&self.body);

        Ok(response)
    }

    pub fn build_headers (
        status_code: u16,
        response_body: &str,
        is_octet: bool
    ) -> HashMap<&'static str, String> {
        let mut headers: HashMap<&'static str, String> = HashMap::new();

        // Build headers and body except for 100, 204, 304 status codes
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
