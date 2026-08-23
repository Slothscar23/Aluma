# Aluma bootstrap

This repository contains a minimal "Aluma" bootstrap that loads HTML/CSS/JS from GitHub and verifies runtime integrity (via an optional manifest) before running. User data is stored locally on the device by default.

How it works:

- The Rust bootstrap fetches runtime files (web/index.html, etc.) from this repository's web/ directory using the repository's default branch.
- If present, web/manifest.json is used to verify SHA256 hashes of runtime files prior to executing them in-memory.
- The HTML is loaded into an in-memory WebView (no runtime files are written to disk by default).
- The page can POST save requests to a local endpoint (http://127.0.0.1:7878/save-local) exposed by the bootstrap; the bootstrap will write files into the user's data directory (XDG/APPDATA or ALUMA_USER_DIR override).
- Optionally, if GITHUB_TOKEN is set in the environment, pages can POST to /save to create/update files in the repo via the GitHub Contents API.

Requirements

- Rust toolchain (cargo)
- For WebView support: platform WebView runtime dependencies (e.g., libgtk-3-dev and libwebkit2gtk-4.0-dev on many Linux distributions; WebView2 runtime on Windows).
- For optional repo saves: set the GITHUB_TOKEN environment variable to a Personal Access Token with repo:contents permissions.

Run

cargo run --release

Security notes

- This prototype executes remote code fetched from the repo — verify integrity in production using manifest signatures and pinning.
- Storing user data in cleartext on disk may be undesirable for highly sensitive information; consider offering encrypted profiles.
- Check platform policies before distributing binaries that download and execute code at runtime.
