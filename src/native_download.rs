//! Shared one-shot native download for already-owned data URLs.
//!
//! Pinned Dioxus alpha.2 MountedData exposes no programmatic click/download
//! method. Its desktop managed-anchor routing (native.ts handleClickNavigate)
//! forwards href without download metadata. Keep this small primary operation
//! for explicit-user-action downloads; Rust owns payloads, names and lifetimes.
//! It is not an alternative rendering, input or DOM-management path.
use serde::{Deserialize, Serialize};

pub(crate) async fn download(data_url: &str, file_name: &str) -> Result<(), String> {
    #[derive(Serialize)]
    struct Request<'a> {
        data_url: &'a str,
        file_name: &'a str,
    }
    #[derive(Deserialize)]
    struct Reply {
        error: Option<String>,
    }
    let task = crate::dom_query::eval(DOWNLOAD);
    task.send(Request {
        data_url,
        file_name,
    })
    .map_err(|error| error.to_string())?;
    let result: Reply = task.join().await.map_err(|error| error.to_string())?;
    result.error.map_or(Ok(()), Err)
}

pub(crate) fn file_name(file_name: &str, extension: &str, default_stem: &str) -> String {
    let clean: String = file_name
        .chars()
        .filter(|c| {
            !c.is_control() && !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        })
        .take(180)
        .collect();
    let clean = clean.trim().trim_matches('.');
    let clean = if clean.is_empty() {
        default_stem
    } else {
        clean
    };
    if clean
        .to_ascii_lowercase()
        .ends_with(&format!(".{extension}"))
    {
        clean.into()
    } else {
        format!("{clean}.{extension}")
    }
}

const DOWNLOAD: &str = r#"
const {data_url, file_name} = await dioxus.recv();
try {
  const link = document.createElement('a'); link.href = data_url; link.download = file_name;
  link.hidden = true; document.body.append(link); link.click(); link.remove(); return {};
} catch (error) { return {error:String(error?.message || error)}; }
"#;
