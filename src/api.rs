use crate::app::{AppMessage, FontAsset, RemoteTheme, SegmentAsset};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Main synchronization task that fetches themes and fonts from GitHub repositories in the background
pub async fn setup_app_task(tx: mpsc::Sender<AppMessage>, themes_dir: PathBuf) {
    let themes_url = "https://api.github.com/repos/JanDeDobbeleer/oh-my-posh/contents/themes";
    let fonts_url = "https://api.github.com/repos/ryanoasis/nerd-fonts/contents/patched-fonts";
    let schema_url = OFFICIAL_SCHEMA_URL;
    setup_app_task_with_urls(tx, themes_dir, themes_url, fonts_url, Some(schema_url)).await;
}

use std::sync::OnceLock;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

pub fn get_client() -> reqwest::Client {
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .user_agent("poshbuddy")
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new())
        })
        .clone()
}

/// Internal helper to check connectivity against a specific address
fn check_internet_connectivity_with_address(address: &str) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    let timeout = std::time::Duration::from_millis(1500);

    match address.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                return TcpStream::connect_timeout(&addr, timeout).is_ok();
            }
        }
        Err(_) => return false,
    }
    false
}

/// Checks if the system has an active internet connection by attempting a fast resolve
pub fn check_internet_connectivity() -> bool {
    // Attempting to resolve a reliable host or connecting to a public DNS
    // We try to connect to a public DNS (Cloudflare) on port 53
    check_internet_connectivity_with_address("1.1.1.1:53")
}

/// Downloads a remote theme file to the local themes directory
pub async fn download_theme_file(
    name: &str,
    url: &str,
    target_dir: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    if !url.starts_with("https://") {
        return Err("Security Error: Only HTTPS URLs are allowed".to_string());
    }
    // Path Traversal Mitigation
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Security Error: Potential path traversal detected in theme name".to_string());
    }
    let client = get_client();
    let file_path = target_dir.join(format!("{}.omp.json", name));

    match client.get(url).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err(format!("Download failed: HTTP {}", resp.status()));
            }
            match resp.bytes().await {
                Ok(bytes) => {
                    if let Err(e) = tokio::fs::write(&file_path, &bytes).await {
                        return Err(format!("Disk write failed: {}", e));
                    }
                    Ok(file_path)
                }
                Err(e) => Err(format!("Network transfer failed: {}", e)),
            }
        }
        Err(e) => Err(format!("Network request failed: {}", e)),
    }
}

/// Downloads a remote theme file to a temporary location for previewing
pub async fn download_to_temp(name: &str, url: &str) -> Result<std::path::PathBuf, String> {
    if !url.starts_with("https://") {
        return Err("Security Error: Only HTTPS URLs are allowed".to_string());
    }
    // Path Traversal Mitigation
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err("Security Error: Potential path traversal detected in theme name".to_string());
    }
    let client = get_client();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download theme for preview: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let temp_dir = std::env::temp_dir().join("poshbuddy_previews");
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| format!("Failed to create preview dir: {}", e))?;

    let file_path = temp_dir.join(format!("{}.omp.json", name));
    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Network transfer failed: {}", e))?;

    tokio::fs::write(&file_path, &bytes)
        .await
        .map_err(|e| format!("Disk write failed: {}", e))?;

    Ok(file_path)
}

pub async fn setup_app_task_with_urls(
    tx: mpsc::Sender<AppMessage>,
    _themes_dir: std::path::PathBuf,
    themes_url: &str,
    fonts_url: &str,
    schema_url: Option<&str>,
) {
    let client = get_client();

    // 1. Fetching available themes from the official Oh My Posh repository
    let resp = client
        .get(themes_url)
        .header("User-Agent", "poshbuddy")
        .send()
        .await;

    if let Ok(r) = resp
        && let Ok(json) = r.json::<serde_json::Value>().await
    {
        // Processing JSON response to extract filenames and download URLs
        let themes: Vec<RemoteTheme> = json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| {
                let name = v["name"].as_str()?.to_string();
                if !name.ends_with(".omp.json") {
                    return None;
                }
                let download_url = v["download_url"].as_str()?.to_string();
                let sha = v["sha"].as_str()?.to_string();
                let clean_name = name.replace(".omp.json", "");
                Some(RemoteTheme {
                    name: clean_name,
                    download_url,
                    sha,
                })
            })
            .collect();

        // Sending the remote themes metadata back to the main UI loop
        if tx
            .send(AppMessage::RemoteThemesLoaded(themes))
            .await
            .is_err()
        {
            return;
        }
    }

    // 2. Fetching Nerd Fonts metadata from the Nerd Fonts repository (patched fonts list)
    let resp_fonts = client
        .get(fonts_url)
        .header("User-Agent", "poshbuddy")
        .send()
        .await;

    if let Ok(r) = resp_fonts
        && let Ok(json) = r.json::<serde_json::Value>().await
    {
        // Filtering for directories that represent different font families
        let fonts: Vec<FontAsset> = json
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter(|v| v["type"] == "dir")
            .filter_map(|v| {
                let name = v["name"].as_str()?.to_string();
                let download_url = v["html_url"].as_str().unwrap_or("").to_string();
                Some(FontAsset { name, download_url })
            })
            .collect();

        // Sending the font metadata back to the main UI loop
        if tx.send(AppMessage::FontsLoaded(fonts)).await.is_err() {}
    }

    // 3. Fetching official Oh My Posh JSON schema to keep segment definitions up to date
    if let Some(s_url) = schema_url {
        let segs_res = fetch_official_segments_from_url(s_url).await;
        if let Ok(official_segments) = segs_res {
            let _ = tx.send(AppMessage::SegmentsLoaded(official_segments)).await;
        }
    }
}

/// URL to official Oh My Posh JSON schema in official repository
pub const OFFICIAL_SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/JanDeDobbeleer/oh-my-posh/main/themes/schema.json";

/// Helper function to parse segment assets directly from official Oh My Posh JSON schema
pub fn parse_segments_from_schema(schema_text: &str) -> Vec<SegmentAsset> {
    let mut segments = Vec::new();
    let Ok(json): Result<serde_json::Value, _> = serde_json::from_str(schema_text) else {
        return segments;
    };

    let defs = json.get("definitions").or_else(|| json.get("$defs"));
    let seg_all_of = defs
        .and_then(|d| d.get("segment"))
        .and_then(|s| s.get("allOf"))
        .and_then(|a| a.as_array());

    if let Some(all_of) = seg_all_of {
        for item in all_of {
            let if_part = item.get("if");
            let then_part = item.get("then");

            let seg_type = if_part
                .and_then(|i| i.get("properties"))
                .and_then(|p| p.get("type"))
                .and_then(|t| t.get("const").or_else(|| t.get("default")))
                .and_then(|v| v.as_str());

            if let Some(st) = seg_type {
                if st == "prompt" || st == "rprompt" {
                    continue;
                }

                let title = then_part
                    .and_then(|t| t.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or(st);

                let description = then_part
                    .and_then(|t| t.get("description"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Official Oh My Posh segment")
                    .to_string();

                let lower_title = title.to_lowercase();
                let category = if lower_title.contains("git")
                    || lower_title.contains("scm")
                    || lower_title.contains("version")
                    || lower_title.contains("fossil")
                    || lower_title.contains("mercurial")
                    || lower_title.contains("subversion")
                {
                    "Version Control"
                } else if lower_title.contains("cloud")
                    || lower_title.contains("docker")
                    || lower_title.contains("aws")
                    || lower_title.contains("azure")
                    || lower_title.contains("kube")
                    || lower_title.contains("gcp")
                    || lower_title.contains("terraform")
                {
                    "Cloud"
                } else if lower_title.contains("lang")
                    || lower_title.contains("node")
                    || lower_title.contains("python")
                    || lower_title.contains("rust")
                    || lower_title.contains("cli")
                    || lower_title.contains("go")
                    || lower_title.contains("java")
                    || lower_title.contains("dotnet")
                    || lower_title.contains("php")
                    || lower_title.contains("ruby")
                {
                    "Development"
                } else if lower_title.contains("time")
                    || lower_title.contains("date")
                    || lower_title.contains("clock")
                {
                    "Time"
                } else if lower_title.contains("package")
                    || lower_title.contains("npm")
                    || lower_title.contains("pnpm")
                    || lower_title.contains("yarn")
                    || lower_title.contains("cargo")
                    || lower_title.contains("composer")
                {
                    "Package Managers"
                } else {
                    "System"
                };

                let clean_name = title
                    .replace(" Segment", "")
                    .replace(" segment", "")
                    .replace(" CLI", "");

                segments.push(SegmentAsset {
                    name: clean_name,
                    segment_type: st.to_string(),
                    description,
                    category: category.to_string(),
                });
            }
        }
    }

    if segments.is_empty() {
        let defs_map_opt = defs.and_then(|v| v.as_object());
        if let Some(defs_map) = defs_map_opt {
            for (key, val) in defs_map {
                let title = match val.get("title").and_then(|t| t.as_str()) {
                    Some(t)
                        if t.to_lowercase().contains("segment") || key.ends_with("_segment") =>
                    {
                        t
                    }
                    _ => continue,
                };
                let seg_type = val
                    .get("properties")
                    .and_then(|p| p.get("type"))
                    .and_then(|t| t.get("const").or_else(|| t.get("default")))
                    .and_then(|v| v.as_str())
                    .unwrap_or_else(|| key.trim_end_matches("_segment"));

                let description = val
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("Official Oh My Posh segment")
                    .to_string();

                segments.push(SegmentAsset {
                    name: title.replace(" Segment", "").replace(" CLI", ""),
                    segment_type: seg_type.to_string(),
                    description,
                    category: "System".to_string(),
                });
            }
        }
    }

    segments.sort_by(|a, b| a.name.cmp(&b.name));
    segments.dedup_by(|a, b| a.segment_type == b.segment_type);
    segments
}

/// Asynchronously fetches the official JSON schema from GitHub to keep segments updated
#[allow(dead_code)]
pub async fn fetch_official_segments() -> Result<Vec<SegmentAsset>, String> {
    fetch_official_segments_from_url(OFFICIAL_SCHEMA_URL).await
}

/// Asynchronously fetches segment assets from a specified schema URL
pub async fn fetch_official_segments_from_url(url: &str) -> Result<Vec<SegmentAsset>, String> {
    let client = get_client();
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch official schema: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Schema fetch HTTP error {}", resp.status()));
    }

    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read schema response: {}", e))?;

    let segments = parse_segments_from_schema(&text);
    if segments.is_empty() {
        Err("No segments parsed from schema".to_string())
    } else {
        Ok(segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use std::path::PathBuf;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_setup_app_task_success() {
        let mut server = Server::new_async().await;

        let theme_mock = server
            .mock("GET", "/themes")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
            [
                {"name": "theme1.omp.json", "type": "file", "download_url": "http://example.com/t1", "sha": "s1"},
                {"name": "theme2.omp.json", "type": "file", "download_url": "http://example.com/t2", "sha": "s2"}
            ]
            "#,
            )
            .create_async()
            .await;

        let font_mock = server
            .mock("GET", "/fonts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
            [
                {"name": "FiraCode", "type": "dir"},
                {"name": "Hack", "type": "dir"}
            ]
            "#,
            )
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url, None).await;

        let msg1 = rx.recv().await.unwrap();
        if let AppMessage::RemoteThemesLoaded(themes) = msg1 {
            assert_eq!(themes.len(), 2);
            assert_eq!(themes[0].name, "theme1");
            assert_eq!(themes[1].name, "theme2");
        } else {
            panic!("Expected RemoteThemesLoaded message");
        }

        let msg2 = rx.recv().await.unwrap();
        if let AppMessage::FontsLoaded(fonts) = msg2 {
            assert_eq!(fonts.len(), 2);
            assert_eq!(fonts[0].name, "FiraCode");
            assert_eq!(fonts[1].name, "Hack");
        } else {
            panic!("Expected FontsLoaded message");
        }

        theme_mock.assert_async().await;
        font_mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_setup_app_task_ignores_invalid_themes() {
        let mut server = Server::new_async().await;

        let _theme_mock = server
            .mock("GET", "/themes")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
            [
                {"name": "theme1.omp.json", "type": "file", "download_url": "http://example.com/t1", "sha": "s1"},
                {"name": "readme.md", "type": "file", "download_url": "http://example.com/r", "sha": "sr"},
                {"name": "theme2.omp.json", "type": "file", "download_url": "http://example.com/t2", "sha": "s2"}
            ]
            "#,
            )
            .create_async()
            .await;

        let _font_mock = server
            .mock("GET", "/fonts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[]"#)
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url, None).await;

        let msg1 = rx.recv().await.unwrap();
        if let AppMessage::RemoteThemesLoaded(themes) = msg1 {
            assert_eq!(themes.len(), 2);
            assert_eq!(themes[0].name, "theme1");
            assert_eq!(themes[1].name, "theme2");
        } else {
            panic!("Expected RemoteThemesLoaded message");
        }
    }

    #[tokio::test]
    async fn test_setup_app_task_ignores_invalid_fonts() {
        let mut server = Server::new_async().await;

        let _theme_mock = server
            .mock("GET", "/themes")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[]"#)
            .create_async()
            .await;

        let _font_mock = server
            .mock("GET", "/fonts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"
            [
                {"name": "FiraCode", "type": "dir"},
                {"name": "SomeFile.ttf", "type": "file"},
                {"name": "Hack", "type": "dir"}
            ]
            "#,
            )
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url, None).await;

        // Wait for ThemesLoaded (empty)
        let _ = rx.recv().await.unwrap();

        let msg2 = rx.recv().await.unwrap();
        if let AppMessage::FontsLoaded(fonts) = msg2 {
            assert_eq!(fonts.len(), 2);
            assert_eq!(fonts[0].name, "FiraCode");
            assert_eq!(fonts[1].name, "Hack");
        } else {
            panic!("Expected FontsLoaded message");
        }
    }

    #[tokio::test]
    async fn test_setup_app_task_handles_http_errors() {
        let mut server = Server::new_async().await;

        let _theme_mock = server
            .mock("GET", "/themes")
            .with_status(500)
            .create_async()
            .await;

        let _font_mock = server
            .mock("GET", "/fonts")
            .with_status(404)
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url, None).await;

        // Channel should be dropped without sending any messages
        assert!(rx.recv().await.is_none());
    }

    #[tokio::test]
    async fn test_setup_app_task_handles_invalid_json() {
        let mut server = Server::new_async().await;

        let _theme_mock = server
            .mock("GET", "/themes")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"invalid": json}"#)
            .create_async()
            .await;

        let _font_mock = server
            .mock("GET", "/fonts")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"<not>json</not>"#)
            .create_async()
            .await;

        let themes_url = format!("{}/themes", server.url());
        let fonts_url = format!("{}/fonts", server.url());

        let (tx, mut rx) = mpsc::channel(10);

        setup_app_task_with_urls(tx, PathBuf::from("dummy"), &themes_url, &fonts_url, None).await;

        // Channel should be dropped without sending any messages
        assert!(rx.recv().await.is_none());
    }

    #[test]
    fn test_check_internet_connectivity_success() {
        let listener =
            std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind to local port");
        let addr = listener.local_addr().expect("Failed to get local address");
        assert!(check_internet_connectivity_with_address(&addr.to_string()));
    }

    #[test]
    fn test_check_internet_connectivity_failure_unbound() {
        // Attempt to connect to an unused port locally (may fail if coincidentally used, but highly unlikely for specific test port)
        assert!(!check_internet_connectivity_with_address("127.0.0.1:54321"));
    }

    #[test]
    fn test_check_internet_connectivity_failure_invalid_address() {
        assert!(!check_internet_connectivity_with_address("invalid:address"));
    }
}

#[cfg(test)]
mod security_tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_download_theme_file_traversal() {
        let themes_dir = PathBuf::from(".");
        let result = download_theme_file("../etc/passwd", "https://example.com", &themes_dir).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path traversal"));
    }

    #[tokio::test]
    async fn test_download_to_temp_traversal() {
        let result = download_to_temp("malicious/path", "https://example.com").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path traversal"));
    }

    #[test]
    fn test_parse_segments_from_schema() {
        let schema_json = r#"{
            "definitions": {
                "segment": {
                    "allOf": [
                        {
                            "if": { "properties": { "type": { "const": "git" } } },
                            "then": { "title": "Git Segment", "description": "Git status" }
                        },
                        {
                            "if": { "properties": { "type": { "const": "node" } } },
                            "then": { "title": "Node CLI Segment", "description": "Node version" }
                        }
                    ]
                }
            }
        }"#;

        let segments = parse_segments_from_schema(schema_json);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].segment_type, "git");
        assert_eq!(segments[0].category, "Version Control");
        assert_eq!(segments[1].segment_type, "node");
        assert_eq!(segments[1].category, "Development");
    }
}
