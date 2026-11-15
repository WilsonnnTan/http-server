pub(crate) struct Parser;

/// Implementation of HTTP/1.1 parser
impl<'a> Parser {
  // Request line format:
  // see: https://www.rfc-editor.org/info/rfc7230/#section-3.1.1

  // Message Parsing Robustness:
  //    Older HTTP/1.0 user agent implementations might send an extra CRLF
  //    after a POST request as a workaround for some early server
  //    applications that failed to read message body content that was not
  //    terminated by a line-ending.  An HTTP/1.1 user agent MUST NOT preface
  //    or follow a request with an extra CRLF.  If terminating the request
  //    message body with a line-ending is desired, then the user agent MUST
  //    count the terminating CRLF octets as part of the message body length.

  //    In the interest of robustness, a server that is expecting to receive
  //    and parse a request-line SHOULD ignore at least one empty line (CRLF)
  //    received prior to the request-line.
  /// return spliited request-line for now
  pub(crate) fn parse(mut request_str: &'a str) -> Vec<&'a str> {
    // ===== Parse request-line =====
    // Ignore at least one empty line (CRLF)
    if let Some(r) = request_str.strip_prefix("\r\n") {
      request_str = r;
    }

    // split request by CRLF, first element will always be request-line
    let splitted_request: Vec<&str> = request_str.split("\r\n").collect();
    // split request-line by SP (single space)
    let splitted_request_line: Vec<&str> = splitted_request[0].split_whitespace().collect();

    splitted_request_line
  }
}