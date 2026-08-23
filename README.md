# Aluma

This repo now includes an APP/ directory that holds the PWA-style application runtime. The intent: keep runtimes on your servers (GitHub), store user data locally (in-browser IndexedDB), and make it clear where the distributable app lives.

What changed
- Added APP/ with a PWA-ready app: index.html, app.js (IndexedDB local storage), style.css, manifest.json, service-worker.js.
- Updated the native bootstrap (src/main.rs) to fetch APP/index.html and verify APP/manifest.json if present.
- The bootstrap still supports optional /save (GitHub) and /save-local (writes to desktop user data dir), but the PWA is designed to store data locally in the browser.

How to use the APP (zero-install via browser)
- You can host the app on GitHub Pages. In repository settings -> Pages, set the source to the main branch and the folder to `/APP` (or keep root and point to /APP/index.html via direct link).
- Visit https://<your-user>.github.io/Aluma/APP/index.html (or the Pages URL) in any modern browser. The app will register a service worker and store user data locally using IndexedDB.
- The "Save to browser storage" button writes to IndexedDB. The optional "Save to repo" button attempts to POST to http://127.0.0.1:7878/save which requires the Rust bootstrap to be running locally and a GITHUB_TOKEN set if you want it to write to the repo.

How to use the native bootstrap (optional)
- The existing Rust bootstrap (aluma-bootstrap) will fetch APP/index.html and load it into an in-memory WebView. It verifies APP/manifest.json entries if present.
- Build and run as before (install Rust toolchain and WebView runtime deps). The bootstrap remains optional — users who want a native binary can download it from Releases (I can set up a GH Actions workflow to produce binaries).

Next recommended steps (pick any):
- I can compute and fill the SHA256 values in APP/manifest.json for the core runtime files and commit that for stronger integrity checks.
- I can add a GitHub Pages configuration or a simple docs/redirect so the app URL is convenient.
- I can add a GitHub Actions workflow to build native release binaries and publish them as Releases.

If you'd like me to compute and commit SHA256 entries now, say "fill manifest" and I'll replace the placeholders with real hex digests for the files in APP/.
