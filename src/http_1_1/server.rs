use super::request::Request;
use super::response::Response;
use anyhow::{Result};
use std::{
    fs::File,
    io::{ErrorKind, Read},
    path::PathBuf,
};

pub(crate) struct Server;

impl<'a> Server {
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


    /// Handles incoming HTTP/1.1 request
    pub(crate) fn handle_request(request_str: &str, directory: &str) -> Result<Vec<u8>> {
        // Parse incoming HTTP/1.1 request and
        // returns response string based on requested route/path
        let (response_status_code, response_body, is_octet) = 
            match Request::parse(request_str) {
                Ok(req) => Self::route(&req, directory),
                Err(e) => {
                    eprintln!("{}", e);
                    (400, String::new(), false)
                }
            };

        // Construct response bytes
        let response_headers = Response::build_headers(
            response_status_code,
            &response_body, 
            is_octet
        );
        let response_str = Response::new(
            response_status_code,
            response_headers,
            response_body
        ).construct_response()?;

        return Ok(response_str.into_bytes());
    }


    /// Handles method and route that returns (status_code, response_body, is_octet)
    fn route(request: &Request, directory: &str) -> (u16, String, bool){
        let empty = |status_code: u16| (status_code, String::new(), false);

        match request.route {
            "/" => empty(200),
            "/user-agent" => {
                match request.headers.get("User-Agent") {
                    Some(&user_agent) => (200, user_agent.to_string(), false),
                    None => empty(400)
                }
            },
            route if let Some(body) = route.strip_prefix("/echo/") => {
                (200, body.to_string(), false)
            },
            route if let Some(file_path) = route.strip_prefix("/files/") => {
                let content = Self::get_file_content(directory, file_path);
                match content {
                    Ok(Some(c)) => (200, c, true),
                    Ok(None) => empty(404),
                    Err(_) => empty(500),
                }
            },
            _ => empty(404),
        }
    }
}

#[cfg(test)]
mod tests {
    
}
