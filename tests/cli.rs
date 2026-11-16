use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Preview {
    child: Child,
    address: String,
}

impl Drop for Preview {
    fn drop(&mut self) {
        self.child.kill().expect("stop preview child");
        self.child.wait().expect("reap preview child");
    }
}

fn request(address: &str, method: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

fn encoding_headers(address: &str, encoding: &str) -> String {
    let mut stream = TcpStream::connect(address).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    write!(stream, "HEAD /demo/_site/sds-lite/sds.css HTTP/1.1\r\nHost: localhost\r\nAccept-Encoding: {encoding}\r\nConnection: close\r\n\r\n").unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[test]
fn cli_preview_serves_watches_and_preserves_a_good_build() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path().join("docs");
    site_generator::create(&root, "Preview test", "https://example.org/demo/").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_sds-site-generator"))
        .arg("preview")
        .arg(&root)
        .args(["--port", "0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut line = String::new();
    stdout.read_line(&mut line).unwrap();
    assert!(line.starts_with("Built "), "{line}");
    line.clear();
    stdout.read_line(&mut line).unwrap();
    let url = url::Url::parse(line.trim().strip_prefix("Preview: ").unwrap()).unwrap();
    let preview = Preview {
        child,
        address: format!("{}:{}", url.host_str().unwrap(), url.port().unwrap()),
    };

    let home = request(&preview.address, "GET", "/demo/");
    assert!(home.starts_with("HTTP/1.1 200"));
    assert!(home.contains("id=\"welcome\""));
    assert!(home.contains("<h1 id=\"welcome\">Welcome</h1>"));
    assert!(home.contains(">Welcome</h1>"));
    assert!(request(&preview.address, "GET", "/").contains("Location: /demo/"));
    assert!(request(&preview.address, "GET", "/demo/guides").contains("Location: /demo/guides/"));
    assert!(request(&preview.address, "GET", "/demo/absent/").starts_with("HTTP/1.1 404"));
    assert!(request(&preview.address, "GET", "/demo/%2e%2e/site.toml").starts_with("HTTP/1.1 400"));
    assert!(
        request(&preview.address, "GET", "/demo/.site-generator-output")
            .starts_with("HTTP/1.1 400")
    );
    assert!(request(&preview.address, "POST", "/demo/").starts_with("HTTP/1.1 405"));
    let head = request(&preview.address, "HEAD", "/demo/");
    assert!(head.starts_with("HTTP/1.1 200"));
    assert!(!head.contains("<!doctype"));
    let gzip = encoding_headers(&preview.address, "br, gzip");
    assert!(gzip.contains("Content-Encoding: gzip"), "{gzip}");
    assert!(gzip.contains("Vary: Accept-Encoding"));
    assert!(!encoding_headers(&preview.address, "gzip;q=0").contains("Content-Encoding"));

    fs::write(
        root.join("content/index.md"),
        "# Updated\n\nFresh content for live preview.",
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if request(&preview.address, "GET", "/demo/").contains("Fresh content for live preview.") {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "Preview did not rebuild after editing Markdown."
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let good = fs::read(root.join("dist/index.html")).unwrap();
    fs::write(
        root.join("content/broken.md"),
        "# Broken\n\n[Missing](missing.md)",
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(1200));
    assert_eq!(fs::read(root.join("dist/index.html")).unwrap(), good);
    assert!(request(&preview.address, "GET", "/demo/").contains("Fresh content for live preview."));
    fs::remove_file(root.join("content/broken.md")).unwrap();
    fs::write(
        root.join("content/index.md"),
        "# Recovered\n\nPreview recovered after an error.",
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        if request(&preview.address, "GET", "/demo/").contains("Preview recovered after an error.")
        {
            break;
        }
        assert!(Instant::now() < deadline, "Preview did not recover.");
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn cli_returns_nonzero_and_explains_invalid_input() {
    let temporary = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_sds-site-generator"))
        .arg("check")
        .arg(temporary.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let diagnostic = String::from_utf8(output.stderr).unwrap();
    assert!(diagnostic.contains("Missing site.toml"), "{diagnostic}");
}
