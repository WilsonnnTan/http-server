use super::parser::Parser;

pub(crate) struct Server;

impl<'a> Server {
  pub(crate) fn handle_request(request_str: &'a str) -> &'static [u8] {
    // TODO: handle four type of request target format https://datatracker.ietf.org/doc/html/rfc9112#section-3.2
    let parsed_request_line = Parser::parse(request_str);

    if parsed_request_line[1] == "/" {
      return b"HTTP/1.1 200 OK\r\n\r\n";
    } else {
      return b"HTTP/1.1 404 Not Found\r\n\r\n";
    }
  }
}