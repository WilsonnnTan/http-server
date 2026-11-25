#[cfg(test)]
mod tests {
    use my_http::tcp::{Client, Listener};
    use reqwest::{header, StatusCode};
    use std::net::SocketAddr;

    #[tokio::test]
    async fn request_with_http_1_1_test() {
        let listener_config = Listener::new(SocketAddr::from(([127, 0, 0, 1], 4222))).unwrap();
        let tcp_listener = listener_config.listen().unwrap();
        std::thread::spawn(|| Client::handle_clients(tcp_listener, ""));

        let (r1, r2, r3) = tokio::join!(
            reqwest::get("http://127.0.0.1:4222"),
            reqwest::get("http://127.0.0.1:4222/abcdefg"),
            reqwest::get("http://127.0.0.1:4222/echo/abcdefg")
        );

        let r1 = r1.unwrap();
        assert_eq!(r1.status(), StatusCode::OK);
        assert_eq!(r1.headers()[header::CONTENT_LENGTH], "0");

        let r2 = r2.unwrap();
        assert_eq!(r2.status(), StatusCode::NOT_FOUND);
        assert_eq!(r2.headers()[header::CONTENT_LENGTH], "0");

        let r3 = r3.unwrap();
        assert_eq!(r3.status(), StatusCode::OK);
        assert_eq!(r3.headers()[header::CONTENT_TYPE], "text/plain");
        assert_eq!(r3.headers()[header::CONTENT_LENGTH], "7");
    }
}
