//! The frontend's files, embedded into the binary. There's no build step, the browser loads
//! the ES modules as they are.

use std::{path::{Component, Path}, sync::OnceLock};

use axum::{http::{HeaderMap, HeaderValue, StatusCode, header}, response::{IntoResponse, Response}};

use crate::InspectorConfig;

macro_rules! assets {
    ($($path:literal),* $(,)?) => {
        &[ $( ($path, include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", $path)) as &[u8]) ),* ]
    };
}

static ASSETS: &[(&str, &[u8])] = assets![
    "index.html",
    "style.css",
    "app.js",
    "api.js",
    "graph.js",
    "panels.js",
    "json-tree.js",
    "util.js",
    "vendor/preact.module.js",
    "vendor/hooks.module.js",
    "vendor/htm.module.js",
    "vendor/signals-core.module.js",
    "vendor/signals.module.js",
    "vendor/cytoscape.esm.min.mjs",
    "vendor/elk.bundled.js",
];

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => "application/octet-stream"
    }
}

/// FNV-1a, the ETags of the embedded files.
fn etag(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("\"{:016x}\"", hash)
}

fn etags() -> &'static [String] {
    static ETAGS: OnceLock<Vec<String>> = OnceLock::new();
    ETAGS.get_or_init(|| ASSETS.iter().map(|(_, bytes)| etag(bytes)).collect())
}

pub(crate) fn serve(config: &InspectorConfig, path: &str, headers: &HeaderMap) -> Response {
    let not_found = || (StatusCode::NOT_FOUND, "Not found").into_response();

    if let Some(ref dir) = config.assets_dir {
        // only plain relative paths
        let relative = Path::new(path);
        if !relative.components().all(|c| matches!(c, Component::Normal(_))) {
            return not_found();
        }
        return match std::fs::read(dir.join(relative)) {
            Ok(bytes) => ([(header::CONTENT_TYPE, content_type(path)), (header::CACHE_CONTROL, "no-store")], bytes).into_response(),
            Err(_) => not_found()
        };
    }

    let Some(i) = ASSETS.iter().position(|(p, _)| *p == path) else { return not_found() };
    let etag = &etags()[i];
    if headers.get(header::IF_NONE_MATCH).and_then(|v| v.to_str().ok()) == Some(etag.as_str()) {
        return StatusCode::NOT_MODIFIED.into_response();
    }

    let mut response = ([(header::CONTENT_TYPE, content_type(path)), (header::CACHE_CONTROL, "no-cache")], ASSETS[i].1).into_response();
    if let Ok(v) = HeaderValue::from_str(etag) {
        response.headers_mut().insert(header::ETAG, v);
    }
    response
}
