use std::io::{BufRead, Read};

pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024; // 10 MB limit
pub const MAX_HEADER_BYTES: usize = 64 * 1024;
pub const BASE_BODY_CAPACITY: usize = 16 * 1024;

pub enum HttpMethod {
    Get,
    Post,
    Options,
    Other,
}

pub struct RequestHead {
    pub method: HttpMethod,
    pub content_length: usize,
    pub keep_alive: bool,
    pub path: String,
    pub accept_html: bool,
}

#[derive(Debug)]
pub enum ParseHeadError {
    Eof,
    HeadersTooLarge,
    BadRequest,
}

pub fn parse_request_head<R: BufRead>(
    reader: &mut R,
    line_buf: &mut String,
) -> Result<RequestHead, ParseHeadError> {
    line_buf.clear();
    match reader
        .by_ref()
        .take((MAX_HEADER_BYTES + 1) as u64)
        .read_line(line_buf)
    {
        Ok(0) => return Err(ParseHeadError::Eof),
        Ok(n) if n > MAX_HEADER_BYTES => return Err(ParseHeadError::HeadersTooLarge),
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to read HTTP request line: {}", e);
            return Err(ParseHeadError::BadRequest);
        }
    }

    let trimmed_req = line_buf.trim();
    if trimmed_req.is_empty() {
        return Err(ParseHeadError::Eof);
    }

    let mut req_parts = trimmed_req.split_whitespace();
    let method_str = req_parts.next().unwrap_or("");
    let path = req_parts.next().unwrap_or("/").to_string();
    let version = req_parts.next().unwrap_or("HTTP/1.1");
    let method = if method_str.eq_ignore_ascii_case("OPTIONS") {
        HttpMethod::Options
    } else if method_str.eq_ignore_ascii_case("GET") {
        HttpMethod::Get
    } else if method_str.eq_ignore_ascii_case("POST") {
        HttpMethod::Post
    } else {
        HttpMethod::Other
    };
    let mut keep_alive = version.eq_ignore_ascii_case("HTTP/1.1");

    let mut content_length = 0usize;
    let mut total_header_bytes = line_buf.len();
    let mut accept_html = false;

    loop {
        line_buf.clear();
        let remaining_cap = (MAX_HEADER_BYTES.saturating_sub(total_header_bytes) + 1) as u64;
        match reader.by_ref().take(remaining_cap).read_line(line_buf) {
            Ok(0) => {
                keep_alive = false;
                break;
            }
            Ok(n) => {
                total_header_bytes += n;
                if total_header_bytes > MAX_HEADER_BYTES {
                    return Err(ParseHeadError::HeadersTooLarge);
                }
                if line_buf.trim().is_empty() {
                    break;
                }
                if let Some((k, v)) = line_buf.split_once(':') {
                    let k_trim = k.trim();
                    if k_trim.eq_ignore_ascii_case("content-length") {
                        content_length = v.trim().parse().unwrap_or(0);
                    } else if k_trim.eq_ignore_ascii_case("connection") {
                        if v.trim().eq_ignore_ascii_case("close") {
                            keep_alive = false;
                        } else if v.trim().eq_ignore_ascii_case("keep-alive") {
                            keep_alive = true;
                        }
                    } else if k_trim.eq_ignore_ascii_case("accept") && v.contains("text/html") {
                        accept_html = true;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read HTTP header line: {}", e);
                return Err(ParseHeadError::BadRequest);
            }
        }
    }

    Ok(RequestHead {
        method,
        content_length,
        keep_alive,
        path,
        accept_html,
    })
}

pub fn read_post_body<R: std::io::Read>(
    reader: &mut R,
    content_length: usize,
    body_buf: &mut Vec<u8>,
) -> Result<(), &'static str> {
    if content_length > MAX_BODY_SIZE {
        return Err("413 Payload Too Large");
    }
    body_buf.resize(content_length, 0);
    if let Err(e) = reader.read_exact(body_buf) {
        body_buf.truncate(0);
        body_buf.shrink_to_fit();
        eprintln!("Failed to read HTTP POST body: {}", e);
        return Err("400 Bad Request");
    }
    Ok(())
}

pub fn reset_body_buffer(body_buf: &mut Vec<u8>) {
    body_buf.truncate(0);
    if body_buf.capacity() > BASE_BODY_CAPACITY {
        body_buf.shrink_to(BASE_BODY_CAPACITY);
    }
}
