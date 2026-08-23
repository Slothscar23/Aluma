use std::env;
use std::thread;
use std::sync::Arc;
use std::time::Duration;
use std::io::Read;
use std::fs;
use std::path::PathBuf;

use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, USER_AGENT, ACCEPT, HeaderMap};
use tiny_http::{Server, Response, Request};
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use serde_json::json;
use anyhow::Result;
use sha2::{Sha256, Digest};
use hex::ToHex;

fn main() -> Result<()> {
    env_logger::init();
    let owner = "Slothscar23".to_string();
    let repo = "Aluma".to_string();

    // Create HTTP client
    let client = Client::builder()
        .user_agent("aluma-bootstrap/0.1")
        .build()?;

    // Determine default branch
    let api_repo = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let repo_meta: serde_json::Value = client.get(&api_repo).send()?.json()?;
    let default_branch = repo_meta.get("default_branch").and_then(|v| v.as_str()).unwrap_or("main");

    // Fetch manifest.json (optional). If missing, fall back to fetching index.html directly.
    let manifest_path = "web/manifest.json";
    let manifest_url = format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, default_branch, manifest_path);

    let mut index_html = String::new();
    let mut manifest_present = false;

    if let Ok(manifest_resp) = client.get(&manifest_url).send() {
        if manifest_resp.status().is_success() {
            if let Ok(manifest_text) = manifest_resp.text() {
                if let Ok(manifest_json) = serde_json::from_str::<serde_json::Value>(&manifest_text) {
                    manifest_present = true;
                    // manifest.files expected to be array of {path, sha256}
                    if let Some(files) = manifest_json.get("files").and_then(|v| v.as_array()) {
                        for file in files {
                            if let Some(path) = file.get("path").and_then(|p| p.as_str()) {
                                let raw_url = format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, default_branch, path);
                                let blob_resp = client.get(&raw_url).send()?;
                                if !blob_resp.status().is_success() {
                                    eprintln!("Failed to fetch {} from repo", path);
                                    continue;
                                }
                                let blob = blob_resp.bytes()?;
                                // compute sha256
                                let mut hasher = Sha256::new();
                                hasher.update(&blob);
                                let result = hasher.finalize();
                                let hexsum = result.encode_hex::<String>();
                                if let Some(expected) = file.get("sha256").and_then(|s| s.as_str()) {
                                    if expected != hexsum {
                                        eprintln!("SHA256 mismatch for {}: expected {} got {}", path, expected, hexsum);
                                        // refuse to run if core runtime mismatches
                                        if path == "web/index.html" {
                                            index_html = format!("<html><body><h1>Integrity check failed</h1><p>Runtime integrity check failed for {}.</p></body></html>", path);
                                            break;
                                        }
                                        continue;
                                    }
                                }

                                if path == "web/index.html" {
                                    index_html = String::from_utf8(blob.to_vec())?;
                                }
                                // For this prototype we do not cache runtime files to disk; they are verified and used in-memory.
                            }
                        }
                    }
                }
            }
        }
    }

    if !manifest_present {
        // fallback: fetch index.html directly
        let index_path = "web/index.html";
        let raw_url = format!("https://raw.githubusercontent.com/{}/{}/{}/{}", owner, repo, default_branch, index_path);
        index_html = client.get(&raw_url).send()?.text()?;
    }

    // Start a local server to accept save requests from the WebView
    let github_token = env::var("GITHUB_TOKEN").ok();
    let owner_arc = Arc::new(owner.clone());
    let repo_arc = Arc::new(repo.clone());
    let client_arc = Arc::new(client);

    let server_owner = owner_arc.clone();
    let server_repo = repo_arc.clone();
    let server_client = client_arc.clone();
    let token_for_thread = github_token.clone();
    thread::spawn(move || {
        let server = Server::http("127.0.0.1:7878").expect("failed to start local server");
        for request in server.incoming_requests() {
            if let Err(e) = handle_request(request, server_client.clone(), server_owner.clone(), server_repo.clone(), token_for_thread.clone()) {
                eprintln!("Error handling request: {}", e);
            }
        }
    });

    // Give server a moment to start
    thread::sleep(Duration::from_millis(200));

    // Launch the WebView with the fetched HTML as a data URL
    use wry::{webview::WebViewBuilder, window::WindowBuilder, application::event_loop::EventLoop, application::event_loop::ControlFlow};

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().with_title("Aluma").build(&event_loop)?;
    let html_base64 = general_purpose::STANDARD.encode(index_html.as_bytes());
    let data_url = format!("data:text/html;charset=utf-8;base64,{}", html_base64);

    let _webview = WebViewBuilder::new(window)?
        .with_url(&data_url)?
        .build()?;

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
    });
}

#[derive(Deserialize)]
struct SaveRequest {
    path: String,
    content: String,
    message: Option<String>,
}

fn user_data_dir() -> PathBuf {
    if let Ok(d) = env::var("ALUMA_USER_DIR") {
        return PathBuf::from(d);
    }

    if cfg!(target_os = "windows") {
        if let Ok(appdata) = env::var("APPDATA") {
            return PathBuf::from(appdata).join("Aluma");
        }
    } else {
        if let Ok(xdg) = env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("aluma");
        }
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(".local").join("share").join("aluma");
        }
    }
    PathBuf::from("user_data")
}

fn handle_request(request: Request, client: Arc<Client>, owner: Arc<String>, repo: Arc<String>, token: Option<String>) -> Result<()> {
    let url = request.url().to_string();
    if url == "/save" && request.method() == &tiny_http::Method::Post {
        // Save to GitHub (requires GITHUB_TOKEN)
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body)?;
        let save: SaveRequest = serde_json::from_str(&body)?;

        if token.is_none() {
            let resp = Response::from_string("GITHUB_TOKEN not set").with_status_code(401);
            request.respond(resp)?;
            return Ok(());
        }
        let token = token.unwrap();

        // Check if file exists to get sha
        let get_url = format!("https://api.github.com/repos/{}/{}/contents/{}", owner, repo, save.path);
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "aluma-bootstrap/0.1".parse().unwrap());
        headers.insert(AUTHORIZATION, format!("token {}", token).parse().unwrap());
        headers.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());

        let get_resp = client.get(&get_url).headers(headers.clone()).send()?;
        let mut sha_opt: Option<String> = None;
        if get_resp.status().is_success() {
            let v: serde_json::Value = get_resp.json()?;
            if let Some(sha) = v.get("sha").and_then(|s| s.as_str()) {
                sha_opt = Some(sha.to_string());
            }
        }

        // Prepare put
        let put_url = get_url;
        let content_b64 = general_purpose::STANDARD.encode(save.content.as_bytes());
        let message = save.message.unwrap_or_else(|| format!("Update {} via Aluma", save.path));
        let mut body_json = json!({
            "message": message,
            "content": content_b64
        });
        if let Some(sha) = sha_opt {
            body_json["sha"] = json!(sha);
        }

        let put_resp = client.put(&put_url)
            .header(USER_AGENT, "aluma-bootstrap/0.1")
            .header(AUTHORIZATION, format!("token {}", token))
            .header(ACCEPT, "application/vnd.github+json")
            .json(&body_json)
            .send()?;

        if put_resp.status().is_success() {
            let resp = Response::from_string("ok");
            request.respond(resp)?;
        } else {
            let resp_text = put_resp.text().unwrap_or_else(|_| "failed".to_string());
            let resp = Response::from_string(resp_text).with_status_code(500);
            request.respond(resp)?;
        }
    } else if url == "/save-local" && request.method() == &tiny_http::Method::Post {
        // Save to local user data directory
        let mut body = String::new();
        request.as_reader().read_to_string(&mut body)?;
        let save: SaveRequest = serde_json::from_str(&body)?;

        let mut base = user_data_dir();
        // sanitize path: prevent escapes outside base
        let rel = PathBuf::from(&save.path);
        if rel.is_absolute() || rel.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            let resp = Response::from_string("invalid path").with_status_code(400);
            request.respond(resp)?;
            return Ok(());
        }
        base.push(rel);
        if let Some(parent) = base.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&base, save.content.as_bytes())?;

        let resp = Response::from_string("ok");
        request.respond(resp)?;
    } else {
        let resp = Response::from_string("not found").with_status_code(404);
        request.respond(resp)?;
    }
    Ok(())
}
