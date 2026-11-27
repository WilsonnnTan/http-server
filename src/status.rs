use std::{collections::HashMap, sync::LazyLock};

pub static STATUS_LOOKUP_TABLE: LazyLock<HashMap<u16, &str>> = LazyLock::new(|| {
    HashMap::from([
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
    ])
});
