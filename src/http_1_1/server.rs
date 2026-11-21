use super::parser::Parser;
use std::collections::HashMap;
use std::fmt::Write;
use anyhow::{Context, Result};

pub(crate) struct Server {
  status_lookup_table: HashMap<i16, &'static str>,
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
  pub(crate) fn build_response(&self, status_code: i16, response_body: &'a str) -> Result<String> {
    let mut response = String::with_capacity(96 + response_body.len());

    // Build status line
    let status_reason = self.status_lookup_table
                          .get(&status_code)
                          .copied()
                          .with_context(|| "Status code not found on status table")?;
    write!(response, "HTTP/1.1 {status_code} {status_reason}\r\n")?;

    // Build headers
    if !response_body.is_empty() {
      // default content-type to text/plain
      response.push_str("Content-Type: text/plain\r\n");
    }
    // keep sending content-length even if it is 0
    response.push_str(&format!("Content-Length: {}\r\n", response_body.len()));
    response.push_str("\r\n"); // end of headers

    // Build Body
    response.push_str(response_body);

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