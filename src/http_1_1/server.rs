use super::request::Request;
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    fmt::Write,
    fs::File,
    io::{ErrorKind, Read},
    path::PathBuf,
};

pub(crate) struct Server {
    status_lookup_table: HashMap<u16, &'static str>,
}

impl<'a> Server {
    pub(crate) fn new() -> Self {
        let status_lookup_table = HashMap::from([
            (100, "Continue"),
            (200, "OK"),
            (201, "Created"),
            (204, "No Content"),
            (301, "Moved Permanently"),
            (302, "Found"),
            (304, "Not Modified"),
            (400, "Bad Request"),
            (401, "Unauthorised"),
            (403, "Forbidden"),
            (404, "Not Found"),
            (429, "Too Many Requests"),
            (500, "Internal Server Error"),
            (502, "Bad Gateway"),
            (503, "Service Unavailable"),
            (504, "Gateway Timeout"),
        ]);

        Self {
            status_lookup_table,
        }
    }

    fn get_file_content(directory: &str, file_path: &str) -> Result<Option<String>> {
        let mut full_path = PathBuf::from(directory);
        full_path.push(file_path);

        let mut content = String::new();

        match File::open(full_path) {
            Ok(mut f) => {
                f.read_to_string(&mut content)?;
                return Ok(Some(content));
            }
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(e.into());
            }
        }
    }

    /// Build HTTP response with text/plain as content-type default
    fn build_response(
        &self,
        status_code: u16,
        response_body: &'a str,
        is_octet: bool,
    ) -> Result<String> {
        let mut response = String::with_capacity(96 + response_body.len());

        // Build status line
        let status_reason = self
            .status_lookup_table
            .get(&status_code)
            .copied()
            .with_context(|| "Status code not found on status table")?;
        write!(response, "HTTP/1.1 {status_code} {status_reason}\r\n")?;

        // Build headers and body except for 100, 204, 304 status codes
        // see: https://www.rfc-editor.org/rfc/rfc9110.html#name-content-length
        if !matches!(status_code, 100u16 | 204u16 | 304u16) {
            if !response_body.is_empty() & !is_octet {
                // default content-type to text/plain
                response.push_str("Content-Type: text/plain\r\n");
            } else if !response_body.is_empty() & is_octet {
                response.push_str("Content-Type: application/octet-stream\r\n");
            }
            // keep sending content-length even if it is 0
            response.push_str(&format!("Content-Length: {}\r\n", response_body.len()));
            response.push_str("\r\n"); // end of headers

            // Build Body
            response.push_str(response_body);
        }

        Ok(response)
    }

    pub(crate) fn handle_request(&self, request_str: &str, directory: &str) -> Result<Vec<u8>> {
        let response: String;

        let parsed_request = match Request::parse(request_str) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("{}", e);
                response = self.build_response(400, "", false)?;
                return Ok(response.into_bytes()); // 400 BAD REQUEST
            }
        };

        let path = parsed_request.path;

        match path {
            "/" => response = self.build_response(200, parsed_request.body, false)?,
            "/user-agent" => {
                response = self.build_response(200, parsed_request.headers["User-Agent"], false)?
            }
            _ if let Some(body) = path.strip_prefix("/echo/") => {
                response = self.build_response(200, body, false)?;
            }
            _ if let Some(file_path) = path.strip_prefix("/files/") => {
                let content = Self::get_file_content(directory, file_path);
                match content {
                    Ok(Some(c)) => response = self.build_response(200, c.as_str(), true)?,
                    Ok(None) => response = self.build_response(404, "", false)?,
                    Err(_) => response = self.build_response(500, "", false)?,
                }
            }
            _ => response = self.build_response(404, parsed_request.body, false)?,
        }

        Ok(response.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    mod response_builder_tests {
        use super::super::Server;
        use std::collections::HashMap;

        #[test]
        /// Test all available status code on status_lookup_table
        fn status_code_test() {
            let status_lookup_table = HashMap::from([
                (100, "Continue"),
                (200, "OK"),
                (201, "Created"),
                (204, "No Content"),
                (301, "Moved Permanently"),
                (302, "Found"),
                (304, "Not Modified"),
                (400, "Bad Request"),
                (401, "Unauthorised"),
                (403, "Forbidden"),
                (404, "Not Found"),
                (429, "Too Many Requests"),
                (500, "Internal Server Error"),
                (502, "Bad Gateway"),
                (503, "Service Unavailable"),
                (504, "Gateway Timeout"),
            ]);

            let server = Server::new();

            for (status_code, status_reason) in status_lookup_table.into_iter() {
                let response = server.build_response(status_code, "", false).unwrap();
                assert!(
                    response.contains(&format!("HTTP/1.1 {} {}\r\n", status_code, status_reason))
                );
            }
        }

        #[test]
        /// Test 100, 204, 304 that doesn't have headers and body
        fn status_code_with_no_content_test() {
            let status_lookup_table = HashMap::from([
                (100, "Continue"),
                (204, "No Content"),
                (304, "Not Modified"),
            ]);

            let server = Server::new();

            for (status_code, status_reason) in status_lookup_table.into_iter() {
                assert_eq!(
                    server.build_response(status_code, "", false).unwrap(),
                    format!("HTTP/1.1 {} {}\r\n", status_code, status_reason)
                );
            }
        }

        #[test]
        #[should_panic(expected = "Status code not found on status table")]
        /// Test for unavailable status code
        fn unavailable_status_code_test() {
            let server = Server::new();
            server.build_response(1001, "", false).unwrap();
        }

        #[test]
        /// Test response builder with status-line, headers, and body
        fn response_builder_test() {
            let server = Server::new();
            assert_eq!(
                server.build_response(200, "abc", false).unwrap(),
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc"
            )
        }
    }

    mod request_handler_tests {
        use super::super::Server;

        #[test]
        fn failed_to_parse_test() {
            let server = Server::new();
            // Failed on checking double CRLF
            assert_eq!(
                server.handle_request("GET / HTTP/1.1\r\n", "").unwrap(),
                b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n"
            );
        }
    }
}
