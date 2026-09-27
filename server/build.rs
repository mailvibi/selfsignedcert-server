use std::{
    env, fs,
    path::{Path, PathBuf},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize)]
struct Manifest {
    assets: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    url: String,
    file: String,
    bytes: usize,
    sha256: String,
    mime: String,
}

fn fail(message: impl AsRef<str>) -> ! {
    panic!("asset staging error: {}", message.as_ref())
}

fn inline_script_hashes(html: &[u8]) -> Vec<String> {
    let html =
        std::str::from_utf8(html).unwrap_or_else(|e| fail(format!("index.html is not UTF-8: {e}")));
    let mut hashes = Vec::new();
    let mut search_from = 0;
    while let Some(relative_start) = html[search_from..].find("<script") {
        let start = search_from + relative_start;
        let open_end = start
            + html[start..]
                .find('>')
                .unwrap_or_else(|| fail("unterminated script tag"))
            + 1;
        let close_start = open_end
            + html[open_end..]
                .find("</script>")
                .unwrap_or_else(|| fail("unterminated script body"));
        let opening_tag = &html[start..open_end];
        if !opening_tag.contains("src=") {
            hashes.push(STANDARD.encode(Sha256::digest(html[open_end..close_start].as_bytes())));
        }
        search_from = close_start + "</script>".len();
    }
    hashes
}

fn main() {
    let staging = env::var_os("SELF_SIGNED_CERT_ASSETS")
        .map(PathBuf::from)
        .unwrap_or_else(|| fail("SELF_SIGNED_CERT_ASSETS is not set; run scripts/release.sh"));
    let manifest_path = staging.join("manifest.json");
    println!("cargo:rerun-if-env-changed=SELF_SIGNED_CERT_ASSETS");
    println!("cargo:rerun-if-changed={}", manifest_path.display());

    let manifest: Manifest = serde_json::from_slice(
        &fs::read(&manifest_path)
            .unwrap_or_else(|e| fail(format!("cannot read {}: {e}", manifest_path.display()))),
    )
    .unwrap_or_else(|e| fail(format!("invalid manifest: {e}")));
    if manifest.assets.is_empty() {
        fail("manifest contains no assets");
    }

    let mut generated = String::from(
        "pub struct Asset { pub path: &'static str, pub bytes: &'static [u8], pub mime: &'static str, pub sha256: &'static str }\npub static ASSETS: &[Asset] = &[\n",
    );
    let mut listed = Vec::new();
    let mut script_hashes = Vec::new();
    for entry in &manifest.assets {
        if !entry.url.starts_with('/')
            || entry.url.contains("..")
            || entry.url.contains('\\')
            || entry.url.contains('%')
        {
            fail(format!("unsafe URL in manifest: {}", entry.url));
        }
        let path = staging.join(&entry.file);
        if !path.starts_with(&staging) || !path.is_file() {
            fail(format!("missing asset: {}", entry.file));
        }
        let bytes = fs::read(&path)
            .unwrap_or_else(|e| fail(format!("cannot read {}: {e}", path.display())));
        if bytes.len() != entry.bytes {
            fail(format!("length mismatch for {}", entry.file));
        }
        if entry.url == "/index.html" {
            script_hashes = inline_script_hashes(&bytes);
        }
        let hash = hex::encode(Sha256::digest(&bytes));
        if hash != entry.sha256 {
            fail(format!("SHA-256 mismatch for {}", entry.file));
        }
        let absolute = path.to_string_lossy().replace('\\', "\\\\");
        generated.push_str(&format!(
            "    Asset {{ path: {:?}, bytes: include_bytes!({:?}), mime: {:?}, sha256: {:?} }},\n",
            entry.url, absolute, entry.mime, entry.sha256
        ));
        listed.push(entry.file.clone());
        println!("cargo:rerun-if-changed={}", path.display());
    }
    let mut actual = fs::read_dir(&staging)
        .unwrap()
        .map(|item| item.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    actual.sort();
    listed.sort();
    let mut expected = listed.clone();
    expected.push("manifest.json".to_string());
    expected.sort();
    if actual != expected {
        fail(format!(
            "staging directory contains unexpected or duplicate files: {actual:?}"
        ));
    }
    generated.push_str("];\n");
    let script_sources = script_hashes
        .iter()
        .map(|hash| format!("'sha256-{hash}'"))
        .collect::<Vec<_>>()
        .join(" ");
    let script_sources = if script_sources.is_empty() {
        String::new()
    } else {
        format!(" {script_sources}")
    };
    let policy = format!(
        "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'{script_sources}; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'"
    );
    generated.push_str(&format!(
        "pub const CONTENT_SECURITY_POLICY: &str = {policy:?};\n"
    ));
    let out = Path::new(&env::var_os("OUT_DIR").unwrap()).join("assets.rs");
    fs::write(out, generated).unwrap();
}
