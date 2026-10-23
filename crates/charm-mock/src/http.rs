//! In-memory HTTP server.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::lock;

/// Recorded request.
#[derive(Clone, Debug)]
pub struct Request {
    /// HTTP method, uppercased.
    pub method: String,
    /// Path without query string.
    pub path: String,
    /// Headers.
    pub headers: BTreeMap<String, String>,
    /// Body as text.
    pub body: String,
}

impl Request {
    /// Returns a header value (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&String> {
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case(name) {
                return Some(v);
            }
        }
        None
    }
}

/// Response writer used by handlers.
#[derive(Clone, Debug, Default)]
pub struct ResponseWriter {
    /// Status code. Zero becomes 200.
    pub status: u16,
    /// Headers.
    pub headers: BTreeMap<String, String>,
    /// Body bytes.
    pub body: Vec<u8>,
}

impl ResponseWriter {
    /// Sets a header.
    pub fn header_set(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }
    /// Writes status code.
    pub fn write_status(&mut self, status: u16) {
        self.status = status;
    }
    /// Writes bytes to body.
    pub fn write(&mut self, data: &[u8]) {
        self.body.extend_from_slice(data);
    }
    /// Writes text to body.
    pub fn write_str(&mut self, s: &str) {
        self.body.extend_from_slice(s.as_bytes());
    }
    /// Returns header value (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&String> {
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case(name) {
                return Some(v);
            }
        }
        None
    }
}

/// Handler type.
pub type Handler = Arc<dyn Fn(&mut ResponseWriter, &Request) + Send + Sync>;

struct HttpInner {
    addr: std::net::SocketAddr,
    listener: Option<TcpListener>,
    closed: bool,
    routes: BTreeMap<String, Handler>,
    requests: Vec<Request>,
}

/// In-memory HTTP server.
#[derive(Clone)]
pub struct HttpMock {
    inner: Arc<Mutex<HttpInner>>,
}

impl std::fmt::Debug for HttpMock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpMock").finish_non_exhaustive()
    }
}

impl Default for HttpMock {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpMock {
    /// Creates and starts the server.
    #[must_use]
    pub fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.set_nonblocking(true).expect("nonblocking");
        let addr = listener.local_addr().expect("addr");
        let inner = Arc::new(Mutex::new(HttpInner {
            addr,
            listener: Some(listener),
            closed: false,
            routes: BTreeMap::new(),
            requests: Vec::new(),
        }));
        let inner_clone = Arc::clone(&inner);
        thread::spawn(move || accept_loop(inner_clone));
        Self { inner }
    }

    /// Base URL.
    pub fn url(&self) -> String {
        let g = lock(&self.inner);
        format!("http://{}", g.addr)
    }

    /// Registers a fixed response.
    pub fn handle(
        &self,
        method: &str,
        path: &str,
        status: i32,
        body: &str,
        headers: Option<BTreeMap<String, String>>,
    ) -> &Self {
        let status = if (100..=999).contains(&status) {
            status as u16
        } else {
            200
        };
        let body = body.to_string();
        let headers = headers.unwrap_or_default();
        let handler: Handler = Arc::new(move |w, _r| {
            for (k, v) in &headers {
                w.header_set(k, v);
            }
            w.write_status(status);
            w.write_str(&body);
        });
        self.handle_func(method, path, Some(handler));
        self
    }

    /// Registers a handler. Pass `None` for a 500 "handler nulo".
    pub fn handle_func(&self, method: &str, path: &str, handler: Option<Handler>) -> &Self {
        let handler = handler.unwrap_or_else(|| {
            Arc::new(|w: &mut ResponseWriter, _r: &Request| {
                w.header_set("Content-Type", "text/plain; charset=utf-8");
                w.write_status(500);
                w.write_str("charm-mock: handler nulo");
            })
        });
        let key = route_key(method, path);
        let mut g = lock(&self.inner);
        g.routes.insert(key, handler);
        // ensure started (already)
        self
    }

    /// Returns a client.
    pub fn client(&self) -> HttpClient {
        HttpClient
    }

    /// Returns a copy of all requests.
    pub fn requests(&self) -> Vec<Request> {
        lock(&self.inner).requests.clone()
    }

    /// Returns `Ok(())` if the route was called.
    pub fn assert_called(&self, method: &str, path: &str) -> Result<(), String> {
        let want = route_key(method, path);
        let requests = self.requests();
        let mut chamadas: Vec<String> = Vec::new();
        for req in &requests {
            if route_key(&req.method, &req.path) == want {
                return Ok(());
            }
            chamadas.push(format!("{} {}", req.method, req.path));
        }
        if requests.is_empty() {
            // still build message
        }
        let lista = if chamadas.is_empty() {
            "(nenhuma)".to_string()
        } else {
            chamadas.join(", ")
        };
        Err(format!(
            "charm-mock: esperava uma chamada {} {}, mas nenhuma ocorreu; chamadas recebidas: {}",
            method.trim().to_uppercase(),
            route_path(path),
            lista
        ))
    }

    /// Closes the server.
    pub fn close(&self) {
        let mut g = lock(&self.inner);
        if g.closed {
            return;
        }
        g.closed = true;
        g.listener = None;
    }
}

/// Minimal HTTP client.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpClient;

impl HttpClient {
    /// GET request.
    pub fn get(&self, url: &str) -> std::io::Result<HttpResponse> {
        self.request("GET", url, &BTreeMap::new(), "")
    }
    /// POST request.
    pub fn post(&self, url: &str, content_type: &str, body: &str) -> std::io::Result<HttpResponse> {
        let mut headers = BTreeMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        self.request("POST", url, &headers, body)
    }
    /// POST with reader: reads all.
    pub fn post_reader(
        &self,
        url: &str,
        content_type: &str,
        body: &str,
    ) -> std::io::Result<HttpResponse> {
        self.post(url, content_type, body)
    }
    /// Generic request with headers and body.
    pub fn request(
        &self,
        method: &str,
        url: &str,
        headers: &BTreeMap<String, String>,
        body: &str,
    ) -> std::io::Result<HttpResponse> {
        let (host, port, path) = parse_url(url)?;
        let mut stream = TcpStream::connect((host.as_str(), port))?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        let mut req = format!("{method} {path} HTTP/1.1\r\nHost: {host}\r\n");
        for (k, v) in headers {
            let _ = write!(req, "{k}: {v}\r\n");
        }
        let _ = write!(req, "Content-Length: {}\r\n", body.len());
        req.push_str("Connection: close\r\n\r\n");
        req.push_str(body);
        stream.write_all(req.as_bytes())?;
        stream.flush()?;

        // read response
        let mut reader = BufReader::new(stream);
        let mut status_line = String::new();
        reader.read_line(&mut status_line)?;
        let mut parts = status_line.split_whitespace();
        let _version = parts.next();
        let status_code: u16 = parts.next().unwrap_or("200").parse().unwrap_or(200);
        let mut headers = BTreeMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line)?;
            let line = line.trim_end_matches(['\r', '\n']).to_string();
            if line.is_empty() {
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                headers.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
        // body: read to EOF or content-length
        let mut body = String::new();
        if let Some(cl) = headers.get("Content-Length") {
            if let Ok(n) = cl.parse::<usize>() {
                let mut buf = vec![0u8; n];
                reader.read_exact(&mut buf)?;
                body = String::from_utf8_lossy(&buf).to_string();
            }
        } else {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf)?;
            body = String::from_utf8_lossy(&buf).to_string();
        }
        Ok(HttpResponse {
            status_code,
            headers,
            body,
        })
    }
}

/// HTTP response.
#[derive(Clone, Debug)]
pub struct HttpResponse {
    /// Status code.
    pub status_code: u16,
    /// Headers.
    pub headers: BTreeMap<String, String>,
    /// Body.
    pub body: String,
}

impl HttpResponse {
    /// Returns a header value (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&String> {
        for (k, v) in &self.headers {
            if k.eq_ignore_ascii_case(name) {
                return Some(v);
            }
        }
        None
    }
}

fn parse_url(url: &str) -> std::io::Result<(String, u16, String)> {
    let url = url.strip_prefix("http://").unwrap_or(url);
    // host:port/path
    let (hostport, path) = match url.find('/') {
        Some(i) => (&url[..i], &url[i..]),
        None => (url, "/"),
    };
    let (host, port) = match hostport.find(':') {
        Some(i) => {
            let port: u16 = hostport[i + 1..]
                .parse()
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad port"))?;
            (hostport[..i].to_string(), port)
        }
        None => (hostport.to_string(), 80),
    };
    let path = if path.is_empty() {
        "/".to_string()
    } else {
        path.to_string()
    };
    Ok((host, port, path))
}

fn route_key(method: &str, path: &str) -> String {
    format!("{} {}", method.trim().to_uppercase(), route_path(path))
}

fn route_path(path: &str) -> String {
    let path = path.split('?').next().unwrap_or(path);
    if path.is_empty() {
        "/".to_string()
    } else if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

#[allow(clippy::needless_pass_by_value)]
fn accept_loop(inner: Arc<Mutex<HttpInner>>) {
    loop {
        let closed = { lock(&inner).closed };
        if closed {
            break;
        }
        let accept_res = {
            let g = lock(&inner);
            if g.closed {
                break;
            }
            match &g.listener {
                Some(l) => l.accept(),
                None => return,
            }
        };
        match accept_res {
            Ok((stream, _)) => {
                let _ = stream.set_nonblocking(false);
                let inner = Arc::clone(&inner);
                thread::spawn(move || handle_connection(inner, stream));
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(2));
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(2));
            }
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_connection(inner: Arc<Mutex<HttpInner>>, mut stream: TcpStream) {
    let Some(req) = read_request(&mut stream) else {
        return;
    };
    let route = {
        let mut g = lock(&inner);
        g.requests.push(req.clone());
        g.routes.get(&route_key(&req.method, &req.path)).cloned()
    };

    let mut writer = ResponseWriter::default();
    if let Some(route) = route {
        route(&mut writer, &req);
    } else {
        writer.status = 404;
        writer.headers.insert(
            "Content-Type".to_string(),
            "text/plain; charset=utf-8".to_string(),
        );
        writer.body.extend_from_slice(
            format!(
                "charm-mock: rota não registrada: {} {}\n",
                req.method, req.path
            )
            .as_bytes(),
        );
    }
    if writer.status == 0 {
        writer.status = 200;
    }
    let _ = write_response(&mut stream, &writer);
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
    // read headers until \r\n\r\n
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match stream.read(&mut byte) {
            Ok(0) => break,
            Ok(_) => {
                buf.push(byte[0]);
                if buf.ends_with(b"\r\n\r\n") {
                    break;
                }
                if buf.len() > 1024 * 1024 {
                    return None;
                }
            }
            Err(_) => return None,
        }
    }
    let header_str = String::from_utf8_lossy(&buf).to_string();
    let mut lines = header_str.split("\r\n");
    let request_line = lines.next()?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let mut headers = BTreeMap::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    let content_length: usize = headers
        .get("Content-Length")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let mut body_bytes = vec![0u8; content_length];
    if content_length > 0 {
        stream.read_exact(&mut body_bytes).ok()?;
    }
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    let path = route_path(&target);
    Some(Request {
        method,
        path,
        headers,
        body,
    })
}

fn write_response(stream: &mut TcpStream, w: &ResponseWriter) -> std::io::Result<()> {
    let status_text = match w.status {
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let mut resp = format!("HTTP/1.1 {} {}\r\n", w.status, status_text);
    for (k, v) in &w.headers {
        let _ = write!(resp, "{k}: {v}\r\n");
    }
    // content-length if not provided
    if !w
        .headers
        .keys()
        .any(|k| k.eq_ignore_ascii_case("Content-Length"))
    {
        let _ = write!(resp, "Content-Length: {}\r\n", w.body.len());
    }
    resp.push_str("Connection: close\r\n\r\n");
    stream.write_all(resp.as_bytes())?;
    stream.write_all(&w.body)?;
    stream.flush()?;
    let _ = stream.shutdown(std::net::Shutdown::Both);
    Ok(())
}
