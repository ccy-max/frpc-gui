//! N7：HTTP 请求检查器（对标 ngrok Traffic Inspector，本地免费实现）
//!
//! 原理：在 127.0.0.1:<port> 起一层本地反向代理，转发到用户的真实服务，
//! 同时捕获每个请求/响应的元数据与（截断展示的）body，存入环形缓冲。
//! 用户把 frpc http 代理的 local_port 指到检查器端口即可获得完整请求日志。
//!
//! v1 限制（有意为之，保证正确性优先）：
//! - 转发追加 `Connection: close`，逐请求短连接（不支持 keep-alive 管线/流式）
//! - 响应体 > 10MB 时截断转发（防 OOM）；展示层各截断 8KB
//! - 不做 TLS MITM（仅 http 明文有效）

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// 展示层 body 截断上限
const BODY_CAP: usize = 8 * 1024;
/// 环形缓冲条数
const BUFFER_CAP: usize = 200;
/// HTTP 头大小上限
const MAX_HEADER_BYTES: usize = 64 * 1024;
/// 转发 body 大小上限（防 OOM）
const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

/// 单条捕获的请求/响应
#[derive(Clone, Serialize)]
pub struct CapturedRequest {
    pub id: u64,
    pub timestamp: i64,
    pub method: String,
    pub path: String,
    pub req_headers: Vec<(String, String)>,
    pub req_body: String,
    pub status: u16,
    pub resp_headers: Vec<(String, String)>,
    pub resp_body: String,
    pub duration_ms: u64,
    pub replayed: bool,
    pub error: Option<String>,
}

/// 检查器状态
pub struct Inspector {
    running: Arc<AtomicBool>,
    next_id: Arc<AtomicU64>,
    requests: Arc<Mutex<VecDeque<CapturedRequest>>>,
    /// 原始报文缓存（重放用）：id → (head_bytes, body_bytes)
    raw: Arc<Mutex<std::collections::HashMap<u64, (Vec<u8>, Vec<u8>)>>>,
    upstream: Arc<Mutex<Option<String>>>,
    port: Arc<Mutex<Option<u16>>>,
}

impl Inspector {
    fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            next_id: Arc::new(AtomicU64::new(1)),
            requests: Arc::new(Mutex::new(VecDeque::new())),
            raw: Arc::new(Mutex::new(std::collections::HashMap::new())),
            upstream: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(None)),
        }
    }
}

/// 全局单例
pub static INSPECTOR: once_cell::sync::Lazy<Arc<Inspector>> =
    once_cell::sync::Lazy::new(|| Arc::new(Inspector::new()));

fn find_double_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

/// 解析 HTTP 头块：返回 (首行, headers)。要求 buf 含完整头（\r\n\r\n 结尾）
fn parse_head(buf: &[u8]) -> Option<(String, Vec<(String, String)>)> {
    let text = std::str::from_utf8(buf).ok()?;
    let end = text.find("\r\n\r\n")?;
    let head = &text[..end];
    let mut lines = head.split("\r\n");
    let first = lines.next()?.to_string();
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    Some((first, headers))
}

fn content_length(headers: &[(String, String)]) -> Option<usize> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.trim().parse::<usize>().ok())
}

/// 读取完整 HTTP 报文。返回 (head 含结尾 CRLFCRLF, headers, body 完整字节)
async fn read_message<S: AsyncReadExt + Unpin>(stream: &mut S) -> std::io::Result<(Vec<u8>, Vec<(String, String)>, Vec<u8>)> {
    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    let mut tmp = [0u8; 4096];
    let head_end;
    loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "连接在读完头之前关闭"));
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_double_crlf(&buf) {
            head_end = pos + 4;
            break;
        }
        if buf.len() > MAX_HEADER_BYTES {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "HTTP 头超过 64KB 上限"));
        }
    }
    let head = buf[..head_end].to_vec();
    let (_, headers) = parse_head(&buf)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "HTTP 头解析失败"))?;
    let cl = content_length(&headers).unwrap_or(0);
    let want = cl.min(MAX_BODY_BYTES);
    let mut body: Vec<u8> = buf[head_end..].to_vec();
    while body.len() < want {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(want);
    Ok((head, headers, body))
}

/// 用状态行/头/完整 body 组装回给客户端的响应（重写 Content-Length + Connection: close）
fn build_response(status: u16, headers: &[(String, String)], body: &[u8]) -> Vec<u8> {
    let reason = match status {
        200 => "OK", 201 => "Created", 204 => "No Content",
        301 => "Moved Permanently", 302 => "Found", 304 => "Not Modified",
        400 => "Bad Request", 401 => "Unauthorized", 403 => "Forbidden",
        404 => "Not Found", 405 => "Method Not Allowed", 409 => "Conflict",
        500 => "Internal Server Error", 502 => "Bad Gateway", 503 => "Service Unavailable",
        _ => "Status",
    };
    let mut out = format!("HTTP/1.1 {} {}\r\n", status, reason).into_bytes();
    for (k, v) in headers {
        if k.eq_ignore_ascii_case("connection") || k.eq_ignore_ascii_case("content-length") {
            continue;
        }
        out.extend_from_slice(format!("{}: {}\r\n", k, v).as_bytes());
    }
    out.extend_from_slice(format!("Content-Length: {}\r\nConnection: close\r\n\r\n", body.len()).as_bytes());
    out.extend_from_slice(body);
    out
}

/// 转发请求到 upstream，返回 (status, headers, 完整 body 字节)
async fn forward_and_capture(upstream: &str, fwd: &[u8]) -> anyhow::Result<(u16, Vec<(String, String)>, Vec<u8>)> {
    let mut up = TcpStream::connect(upstream)
        .await
        .map_err(|e| anyhow::anyhow!("连接上游 {} 失败: {}", upstream, e))?;
    up.write_all(fwd).await?;
    up.flush().await?;
    let (resp_head, resp_headers, resp_body) = read_message(&mut up).await?;
    let (status_line, _) = parse_head(&resp_head)
        .ok_or_else(|| anyhow::anyhow!("响应头解析失败"))?;
    let status = status_line
        .splitn(3, ' ')
        .nth(1)
        .and_then(|s| s.trim().parse::<u16>().ok())
        .unwrap_or(0);
    Ok((status, resp_headers, resp_body))
}

/// 重建转发报文：原 head（去掉 Connection）+ Connection: close + body
fn build_forward(head: &[u8], headers: &[(String, String)], body: &[u8]) -> Vec<u8> {
    let mut fwd = Vec::with_capacity(head.len() + body.len() + 32);
    // head 以 \r\n\r\n 结尾；去掉末尾一个 CRLF 以便追加头，再补空行
    fwd.extend_from_slice(&head[..head.len().saturating_sub(2)]);
    if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("connection")) {
        fwd.extend_from_slice(b"Connection: close\r\n");
    }
    fwd.extend_from_slice(b"\r\n");
    fwd.extend_from_slice(body);
    fwd
}

/// 处理单个连接
async fn handle_conn(insp: Arc<Inspector>, mut client: TcpStream, upstream: String) {
    let started = std::time::Instant::now();
    let (head, req_headers, body) = match read_message(&mut client).await {
        Ok(v) => v,
        Err(e) => {
            log::debug!("[inspector] 读请求失败: {}", e);
            return;
        }
    };
    let (first_line, _) = match parse_head(&head) {
        Some(v) => v,
        None => return,
    };
    let mut parts = first_line.splitn(3, ' ');
    let method = parts.next().unwrap_or("?").to_string();
    let path = parts.next().unwrap_or("/").to_string();

    let id = insp.next_id.fetch_add(1, Ordering::SeqCst);
    {
        let mut raw = insp.raw.lock().await;
        raw.insert(id, (head.clone(), body.clone()));
        // 防泄漏：超过 2× 缓冲上限时清最旧
        if raw.len() > BUFFER_CAP * 2 {
            let mut keys: Vec<u64> = raw.keys().copied().collect();
            keys.sort();
            let drop_n = raw.len() - BUFFER_CAP;
            for k in keys.into_iter().take(drop_n) {
                raw.remove(&k);
            }
        }
    }

    let fwd = build_forward(&head, &req_headers, &body);
    let mut captured = CapturedRequest {
        id,
        timestamp: chrono::Local::now().timestamp(),
        method,
        path,
        req_headers,
        req_body: String::from_utf8_lossy(&body[..body.len().min(BODY_CAP)]).to_string(),
        status: 0,
        resp_headers: Vec::new(),
        resp_body: String::new(),
        duration_ms: 0,
        replayed: false,
        error: None,
    };

    match forward_and_capture(&upstream, &fwd).await {
        Ok((status, resp_headers, resp_body)) => {
            captured.status = status;
            captured.resp_headers = resp_headers.clone();
            captured.resp_body =
                String::from_utf8_lossy(&resp_body[..resp_body.len().min(BODY_CAP)]).to_string();
            let out = build_response(status, &resp_headers, &resp_body);
            let _ = client.write_all(&out).await;
        }
        Err(e) => {
            captured.error = Some(e.to_string());
            let msg = format!("upstream error: {}", e);
            let resp = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                msg.len(),
                msg
            );
            let _ = client.write_all(resp.as_bytes()).await;
        }
    }
    captured.duration_ms = started.elapsed().as_millis() as u64;

    let mut q = insp.requests.lock().await;
    q.push_back(captured);
    while q.len() > BUFFER_CAP {
        q.pop_front();
    }
}

// ==================== 对外命令实现 ====================

pub async fn start(port: u16, upstream: String) -> Result<(), String> {
    if INSPECTOR.running.load(Ordering::SeqCst) {
        return Err("检查器已在运行，请先停止".to_string());
    }
    let listener = TcpListener::bind(("127.0.0.1", port))
        .await
        .map_err(|e| format!("监听 127.0.0.1:{} 失败: {}", port, e))?;
    *INSPECTOR.upstream.lock().await = Some(upstream.clone());
    *INSPECTOR.port.lock().await = Some(port);
    INSPECTOR.running.store(true, Ordering::SeqCst);
    let insp = INSPECTOR.clone();
    log::info!("[inspector] started on :{} → {}", port, upstream);
    tokio::spawn(async move {
        while insp.running.load(Ordering::SeqCst) {
            match tokio::time::timeout(std::time::Duration::from_millis(800), listener.accept()).await {
                Ok(Ok((stream, _))) => {
                    let insp2 = insp.clone();
                    let up = insp2.upstream.lock().await.clone().unwrap_or_default();
                    if up.is_empty() {
                        continue;
                    }
                    tokio::spawn(async move {
                        handle_conn(insp2, stream, up).await;
                    });
                }
                Ok(Err(e)) => {
                    log::warn!("[inspector] accept error: {}", e);
                    break;
                }
                Err(_) => continue, // timeout → 重新检查 running
            }
        }
        log::info!("[inspector] accept loop exited");
    });
    Ok(())
}

pub async fn stop() -> Result<(), String> {
    INSPECTOR.running.store(false, Ordering::SeqCst);
    *INSPECTOR.upstream.lock().await = None;
    *INSPECTOR.port.lock().await = None;
    Ok(())
}

pub async fn status() -> Result<(bool, Option<u16>, Option<String>), String> {
    Ok((
        INSPECTOR.running.load(Ordering::SeqCst),
        *INSPECTOR.port.lock().await,
        INSPECTOR.upstream.lock().await.clone(),
    ))
}

pub async fn list_requests() -> Result<Vec<CapturedRequest>, String> {
    Ok(INSPECTOR.requests.lock().await.iter().cloned().collect())
}

pub async fn clear_requests() -> Result<(), String> {
    INSPECTOR.requests.lock().await.clear();
    INSPECTOR.raw.lock().await.clear();
    Ok(())
}

/// 重放：取出原始报文重新发往 upstream，结果作为新条目（replayed=true）
pub async fn replay(id: u64) -> Result<u64, String> {
    let (head, body) = INSPECTOR
        .raw
        .lock()
        .await
        .get(&id)
        .cloned()
        .ok_or_else(|| format!("请求 {} 的原始报文已过期（缓冲上限 {}）", id, BUFFER_CAP * 2))?;
    let upstream = INSPECTOR
        .upstream
        .lock()
        .await
        .clone()
        .ok_or_else(|| "检查器未运行，无法重放".to_string())?;
    let (first_line, req_headers) = parse_head(&head).ok_or_else(|| "原始头解析失败".to_string())?;
    let mut parts = first_line.splitn(3, ' ');
    let method = parts.next().unwrap_or("?").to_string();
    let path = parts.next().unwrap_or("/").to_string();

    let started = std::time::Instant::now();
    let fwd = build_forward(&head, &req_headers, &body);
    let new_id = INSPECTOR.next_id.fetch_add(1, Ordering::SeqCst);
    let mut captured = CapturedRequest {
        id: new_id,
        timestamp: chrono::Local::now().timestamp(),
        method,
        path,
        req_headers,
        req_body: String::from_utf8_lossy(&body[..body.len().min(BODY_CAP)]).to_string(),
        status: 0,
        resp_headers: Vec::new(),
        resp_body: String::new(),
        duration_ms: 0,
        replayed: true,
        error: None,
    };
    match forward_and_capture(&upstream, &fwd).await {
        Ok((status, resp_headers, resp_body)) => {
            captured.status = status;
            captured.resp_headers = resp_headers;
            captured.resp_body =
                String::from_utf8_lossy(&resp_body[..resp_body.len().min(BODY_CAP)]).to_string();
        }
        Err(e) => captured.error = Some(e.to_string()),
    }
    captured.duration_ms = started.elapsed().as_millis() as u64;
    let mut q = INSPECTOR.requests.lock().await;
    q.push_back(captured);
    while q.len() > BUFFER_CAP {
        q.pop_front();
    }
    Ok(new_id)
}
