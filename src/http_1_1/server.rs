use super::parser::Parser;
use std::collections::HashMap;
use std::fmt::Write;
use anyhow::{Context, Result};

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
      (504, "Gateway Timeout")
    ]);

    Self {
      status_lookup_table
    }
  }

  /// Build HTTP response with text/plain as content-type default
  fn build_response(&self, status_code: u16, response_body: &'a str) -> Result<String> {
    let mut response = String::with_capacity(96 + response_body.len());

    // Build status line
    let status_reason = self.status_lookup_table
                          .get(&status_code)
                          .copied()
                          .with_context(|| "Status code not found on status table")?;
    write!(response, "HTTP/1.1 {status_code} {status_reason}\r\n")?;

    // Build headers and body except for 100, 204, 304 status codes
    match matches!(status_code, 100u16 | 204u16 | 304u16) {
      false => {
        if !response_body.is_empty() {
          // default content-type to text/plain
          response.push_str("Content-Type: text/plain\r\n");
        }
        // keep sending content-length even if it is 0
        response.push_str(&format!("Content-Length: {}\r\n", response_body.len()));
        response.push_str("\r\n"); // end of headers

        // Build Body
        response.push_str(response_body);
      },
      true => {}
    }

    Ok(response)
  }

  pub(crate) fn handle_request(&self, request_str: &'a str) -> Vec<u8> {
    // TODO: handle four type of request target format https://datatracker.ietf.org/doc/html/rfc9112#section-3.2
    let parsed_request = Parser::parse(request_str);

    // split request-line by SP (single space)
    let splitted_request_line: Vec<&str> = parsed_request[0].split_whitespace().collect();

    if splitted_request_line[1] == "/" {
      return self.build_response(200, "").unwrap().into_bytes();
    } else if let Some(s) = splitted_request_line[1].strip_prefix("/echo/") {
      return self.build_response(200, s).unwrap().into_bytes();
    } else {
      return self.build_response(404, "").unwrap().into_bytes();
    }
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
        (504, "Gateway Timeout")
      ]);
      
      let server = Server::new();

      for (status_code, status_reason) in status_lookup_table.into_iter() {
        let response = server.build_response(status_code, "").unwrap();
        assert!(response.contains(&format!("HTTP/1.1 {} {}\r\n", status_code, status_reason)));
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
          server.build_response(status_code, "").unwrap(),
          format!("HTTP/1.1 {} {}\r\n", status_code, status_reason)
        );
      }
    }

    #[test]
    #[should_panic(expected = "Status code not found on status table")]
    /// Test for unavailable status code
    fn unavailable_status_code_test() {
      let server = Server::new();
      server.build_response(1001, "").unwrap();
    }

    #[test]
    /// Test response builder with status-line, headers, and body
    fn response_builder_test() {
      let server = Server::new();
      assert_eq!(
        server.build_response(200, "abc").unwrap(),
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 3\r\n\r\nabc"
      )
    }
  }
}