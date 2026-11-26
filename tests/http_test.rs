#[cfg(test)]
mod tests {
    use reqwest::{header, StatusCode};
    use std::{io::Write, process};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn request_with_http_1_1_test() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "This is temp file.").unwrap();
        let file_path = file.path();
        let file_dir = file_path.parent().unwrap().to_str().unwrap();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        let mut child = process::Command::new(env!("CARGO_BIN_EXE_http-server"))
            .args(["--directory", file_dir])
            .spawn()
            .expect("Failed to spawn HTTP server.");

        let (r1, r2, r3, r4, r5) = tokio::join!(
            reqwest::get("http://127.0.0.1:4222"),
            reqwest::get("http://127.0.0.1:4222/abcdefg"),
            reqwest::get("http://127.0.0.1:4222/echo/abcdefg"),
            reqwest::get("http://127.0.0.1:4222/files/non_existant_file"),
            reqwest::get(format!("http://127.0.0.1:4222/files/{file_name}")),
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

        let r4 = r4.unwrap();
        assert_eq!(r4.status(), StatusCode::NOT_FOUND);
        assert_eq!(r4.headers()[header::CONTENT_LENGTH], "0");

        let r5 = r5.unwrap();
        assert_eq!(r5.status(), StatusCode::OK);
        assert_eq!(r5.headers()[header::CONTENT_LENGTH], "18");
        assert_eq!(
            r5.headers()[header::CONTENT_TYPE],
            "application/octet-stream"
        );
        assert_eq!(r5.text().await.unwrap(), "This is temp file.");

        child.kill().expect("Failed to kill HTTP server.");
        child.wait().expect("Failed to wait on server");
    }
}
