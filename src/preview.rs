use crate::{build, config::Config, content::files};
use anyhow::{Context, Result, bail, ensure};
use percent_encoding::percent_decode_str;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Component, Path};
use std::time::{Duration, Instant};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

pub fn preview(directory: &Path, port: u16) -> Result<()> {
    let root = directory.canonicalize()?;
    let report = build(&root)?;
    print_warnings(&report);
    let server = Server::http(("127.0.0.1", port))
        .map_err(|error| anyhow::anyhow!("Could not start preview: {error}"))?;
    let address = server.server_addr().to_string();
    let mut config = Config::load(&root)?;
    println!(
        "Built {} pages.\nPreview: http://{}{}\nWatching content/, public/, and site.toml. Refresh after saving. Ctrl+C to stop.",
        report.pages,
        address,
        config.url.path()
    );
    let mut fingerprint = snapshot(&root)?;
    let mut last_scan = Instant::now();
    let mut last_error = String::new();
    loop {
        if last_scan.elapsed() >= Duration::from_millis(500) {
            last_scan = Instant::now();
            match snapshot(&root) {
                Ok(next) if next != fingerprint => {
                    fingerprint = next;
                    match build(&root) {
                        Ok(report) => {
                            print_warnings(&report);
                            config = Config::load(&root)?;
                            last_error.clear();
                            println!(
                                "Rebuilt {} pages. Refresh http://{}{}",
                                report.pages,
                                address,
                                config.url.path()
                            );
                        }
                        Err(error) => {
                            last_error = format!("{error:#}");
                            eprintln!(
                                "Build failed; still serving the last good build:\n{last_error}"
                            );
                        }
                    }
                }
                Err(error) => {
                    let message = format!("{error:#}");
                    if message != last_error {
                        eprintln!(
                            "Could not watch source files; still serving the last good build:\n{message}"
                        );
                        last_error = message;
                    }
                }
                _ => {}
            }
        }
        if let Some(request) = server.recv_timeout(Duration::from_millis(100))?
            && let Err(error) = serve(request, &report.output, config.url.path())
        {
            eprintln!("Preview request failed: {error:#}");
        }
    }
}

fn print_warnings(report: &crate::BuildReport) {
    for warning in report.warnings.iter().take(20) {
        eprintln!(
            "Warning: {}:{}: {}",
            warning.file, warning.line, warning.message
        );
    }
    if !report.warnings.is_empty() {
        eprintln!(
            "{} build warnings. Full report: {}",
            report.warnings.len(),
            report.output.join("_site/diagnostics.json").display()
        );
    }
}

fn snapshot(root: &Path) -> Result<u64> {
    let mut hasher = DefaultHasher::new();
    for path in std::iter::once(root.join("site.toml"))
        .chain(files(&root.join("content"))?)
        .chain(if root.join("public").exists() {
            files(&root.join("public"))?
        } else {
            Vec::new()
        })
    {
        path.hash(&mut hasher);
        fs::read(&path)
            .with_context(|| format!("Watch {}", path.display()))?
            .hash(&mut hasher);
    }
    Ok(hasher.finish())
}

fn serve(request: Request, output: &Path, base: &str) -> Result<()> {
    let head = request.method() == &Method::Head;
    if !head && request.method() != &Method::Get {
        return reply(
            request,
            405,
            "text/plain; charset=utf-8",
            b"Only GET and HEAD are supported.".to_vec(),
            false,
            None,
        );
    }
    let raw = request.url().split('?').next().unwrap_or("/");
    let decoded = match percent_decode_str(raw).decode_utf8() {
        Ok(path) => path,
        Err(_) => {
            return reply(
                request,
                400,
                "text/plain; charset=utf-8",
                b"Invalid URL encoding.".to_vec(),
                head,
                None,
            );
        }
    };
    if decoded == "/" && base != "/" {
        return reply(request, 302, "text/plain", Vec::new(), head, Some(base));
    }
    let Some(relative) = decoded.strip_prefix(base) else {
        return reply(
            request,
            404,
            "text/plain; charset=utf-8",
            b"This URL is outside the preview site.".to_vec(),
            head,
            None,
        );
    };
    if relative.contains('\\')
        || relative.contains('\0')
        || Path::new(relative)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || relative.split('/').any(|part| part.starts_with('.'))
    {
        return reply(
            request,
            400,
            "text/plain; charset=utf-8",
            b"Invalid path.".to_vec(),
            head,
            None,
        );
    }
    let mut path = output.join(relative);
    if path.is_dir() {
        if !decoded.ends_with('/') {
            let location = format!("{decoded}/");
            return reply(
                request,
                301,
                "text/plain",
                Vec::new(),
                head,
                Some(&location),
            );
        }
        path = path.join("index.html");
    }
    let exists = path.is_file();
    let status = if exists { 200 } else { 404 };
    if !exists {
        path = output.join("404.html");
    }
    let canonical = path.canonicalize()?;
    ensure!(
        canonical.starts_with(output.canonicalize()?),
        "Refusing to serve a file outside dist."
    );
    let mime = match path.extension().and_then(|s| s.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "xml" => "application/xml",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "pdf" => "application/pdf",
        "woff2" => "font/woff2",
        "csv" => "text/csv; charset=utf-8",
        "zip" => "application/zip",
        "tif" | "tiff" => "image/tiff",
        "jar" => "application/java-archive",
        "tar" => "application/x-tar",
        "gz" => "application/gzip",
        _ => "text/plain; charset=utf-8",
    };
    let accepts_gzip = request
        .headers()
        .iter()
        .find(|header| header.field.equiv("Accept-Encoding"))
        .is_some_and(|header| {
            header.value.as_str().split(',').any(|encoding| {
                let mut parts = encoding.trim().split(';');
                parts
                    .next()
                    .is_some_and(|part| part.trim().eq_ignore_ascii_case("gzip"))
                    && parts.all(|part| {
                        let part = part.trim();
                        !part.starts_with("q=")
                            || part
                                .strip_prefix("q=")
                                .and_then(|q| q.parse::<f32>().ok())
                                .is_some_and(|q| q > 0.0 && q <= 1.0)
                    })
            })
        });
    let gzip = canonical.with_file_name(format!(
        "{}.gz",
        canonical
            .file_name()
            .context("Requested file has no name.")?
            .to_string_lossy()
    ));
    if accepts_gzip && gzip.is_file() {
        let gzip = gzip.canonicalize()?;
        ensure!(
            gzip.starts_with(output.canonicalize()?),
            "Refusing to serve compressed content outside dist."
        );
        return reply_encoded(request, status, mime, fs::read(gzip)?, head, None, true);
    }
    reply(request, status, mime, fs::read(canonical)?, head, None)
}

fn reply(
    request: Request,
    status: u16,
    mime: &str,
    bytes: Vec<u8>,
    head: bool,
    location: Option<&str>,
) -> Result<()> {
    reply_encoded(request, status, mime, bytes, head, location, false)
}

fn reply_encoded(
    request: Request,
    status: u16,
    mime: &str,
    bytes: Vec<u8>,
    head: bool,
    location: Option<&str>,
    gzip: bool,
) -> Result<()> {
    let length = bytes.len();
    let mut response = Response::from_data(if head { Vec::new() } else { bytes })
        .with_status_code(StatusCode(status));
    for (name, value) in [
        ("Content-Type", mime),
        ("Cache-Control", "no-store"),
        ("X-Content-Type-Options", "nosniff"),
        ("Referrer-Policy", "strict-origin-when-cross-origin"),
        ("Vary", "Accept-Encoding"),
    ] {
        response.add_header(header(name, value)?);
    }
    if head {
        response.add_header(header("Content-Length", &length.to_string())?);
    }
    if gzip {
        response.add_header(header("Content-Encoding", "gzip")?);
    }
    if let Some(location) = location {
        response.add_header(header("Location", location)?);
    }
    request.respond(response)?;
    Ok(())
}

fn header(name: &str, value: &str) -> Result<Header> {
    match Header::from_bytes(name.as_bytes(), value.as_bytes()) {
        Ok(header) => Ok(header),
        Err(()) => bail!("Invalid HTTP header {name}."),
    }
}
