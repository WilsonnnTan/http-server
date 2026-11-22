use std::{
  collections::HashMap,
  str::FromStr,
};
use anyhow::{Result, bail};


#[derive(Debug, PartialEq, Eq)]
/// HTTP/1.1 request methods
pub (crate) enum Method {
  GET,
  POST,
  PATCH,
  PUT,
  DELETE,
}

/// FromStr implementation to parse str into Method enum
impl FromStr for Method {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self>{
    match s {
      "GET" => Ok(Method::GET),
      "POST" => Ok(Method::POST),
      "PATCH" => Ok(Method::PATCH),
      "PUT" => Ok(Method::PUT),
      "DELETE" =>  Ok(Method::DELETE),
      other => bail!("Unsupported method: {}", other),
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Request<'a> {
  /// HTTP request method
  pub(crate) method: Method,
  /// HTTP request path
  pub(crate) path: &'a str,
  /// HTTP headers
  pub(crate) headers: HashMap<String, &'a str>,
  /// HTTP body
  pub(crate) body: &'a str,
}

/// Request FromStr implementation
impl<'a> Request<'a> {
  // HTTP/1.1 format: https://www.rfc-editor.org/info/rfc9110
  /// Returns request object wrapped in anyhow::Result
  /// that is parsed from incoming HTTP/1.1 request
  pub(crate) fn parse(mut request_str: &'a str) -> Result<Self> {
    // Ignore at least one empty line (CRLF)
    if let Some(r) = request_str.strip_prefix("\r\n") {
      request_str = r;
    }

    // Split body from request_str by double CRLF (end of headers)
    let Some((reqline_and_headers, body)) = request_str.split_once("\r\n\r\n")
    else { bail!("failed to parse HTTP request.") }; // 400 BAD REQUEST

    // Split request line and headers by single CRLF
    let Some((request_line, headers)) = reqline_and_headers.split_once("\r\n")
    else { bail!("failed to parse HTTP request.") }; // 400 BAD REQUEST


    // Parse request_line
    let splitted_request_line: Vec<&str> = request_line.split(" ").collect();
    let request_method: Method = Method::from_str(splitted_request_line[0])?;
    let request_path = splitted_request_line[1];

    // Parse headers
    let mut parsed_headers: HashMap<String, &str> = HashMap::new();
    let splitted_headers: Vec<&str> = headers.split("\r\n").collect();
    for header in splitted_headers {
      let Some((field, value)) = header.split_once(":")
      else { bail!("failed to parse HTTP request.") }; // 400 BAD REQUEST

      parsed_headers.insert(field.to_lowercase(), value.trim());
    }

    Ok(
      Self {
        method: request_method,
        path: request_path,
        headers: parsed_headers,
        body: body,
      }
    )
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  /// Test parser to successfully parse status-line, headers, and body
  fn http_1_1_parse_test() {
    assert_eq!(
      Request::parse("POST / HTTP/1.1\r\n\
        Host: localhost:4222\r\n\
        User-Agent: curl/8.22.0\r\n\
        Accept: */*\r\n\
        Content-Length: 2\r\n\
        Content-Type: application/x-www-form-urlencoded\r\n\
        \r\n\
        hi"
      ).unwrap(),
      Request {
        method: Method::POST,
        path: "/",
        headers: HashMap::from([
                  ("host".to_string(), "localhost:4222"),
                  ("user-agent".to_string(), "curl/8.22.0"),
                  ("accept".to_string(), "*/*"),
                  ("content-length".to_string(), "2"),
                  ("content-type".to_string(), "application/x-www-form-urlencoded"),
                ]),
        body: "hi",
      }
    );
  }
}
