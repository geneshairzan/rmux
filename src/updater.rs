//! On-demand update check and install for rmux.
//!
//! No update happens on its own: everything here runs only when the user
//! presses "Check for Updates" in Settings → About. Network access is a
//! blocking `curl` subprocess against the GitHub releases API, so no new
//! HTTP crate lands in the binary. Installs go through `pkexec dpkg -i`,
//! which always shows the system authentication dialog.

/// Repository the update check queries. Kept in one place so a fork only
/// changes this line.
const GITHUB_REPO: &str = "geneshairzan/rmux";

fn latest_release_api_url() -> String {
    format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest")
}

/// Release information the UI needs. Plain data, never executed.
pub struct ReleaseInfo {
    pub version: String,
    pub page_url: String,
    pub deb_asset_url: Option<String>,
}

/// States the Settings About page walks through.
pub enum UpdateOutcome {
    UpToDate(String),
    Available(Box<ReleaseInfo>),
    Failed(String),
}

/// Numeric `major.minor.patch` comparison, tolerant of a leading `v` and
/// of missing components (`1.0` equals `1.0.0`).
pub fn is_newer_than(current: &str, latest: &str) -> bool {
    fn parts(v: &str) -> Vec<u64> {
        v.trim()
            .trim_start_matches('v')
            .split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect()
    }
    let cur = parts(current);
    let lat = parts(latest);
    for i in 0..cur.len().max(lat.len()) {
        let c = cur.get(i).copied().unwrap_or(0);
        let l = lat.get(i).copied().unwrap_or(0);
        if l != c {
            return l > c;
        }
    }
    false
}

fn curl_text(url: &str, max_time_secs: u64) -> Result<String, String> {
    let out = std::process::Command::new("curl")
        .args([
            "-sL",
            "--max-time",
            &max_time_secs.to_string(),
            "-H",
            "Accept: application/vnd.github+json",
            url,
        ])
        .output()
        .map_err(|e| format!("could not run curl: {e}"))?;
    if !out.status.success() {
        let detail = String::from_utf8_lossy(&out.stderr);
        return Err(format!("download failed: {}", detail.trim()));
    }
    String::from_utf8(out.stdout).map_err(|_| "response was not valid text".to_string())
}

/// Query the GitHub latest-release endpoint and extract the tag, page URL
/// and the amd64 `.deb` asset URL when one exists.
pub fn fetch_latest() -> Result<ReleaseInfo, String> {
    let body = curl_text(&latest_release_api_url(), 15)?;
    let json: serde_json::Value =
        serde_json::from_str(&body).map_err(|_| "could not parse release response".to_string())?;
    let tag = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "release response has no version".to_string())?;
    let page_url = json
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let mut deb_asset_url = None;
    if let Some(assets) = json.get("assets").and_then(|v| v.as_array()) {
        for asset in assets {
            let name = asset
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if name.ends_with("_amd64.deb") {
                deb_asset_url = asset
                    .get("browser_download_url")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                break;
            }
        }
    }
    Ok(ReleaseInfo {
        version: tag.to_string(),
        page_url,
        deb_asset_url,
    })
}

pub fn check_for_updates() -> UpdateOutcome {
    let current = env!("CARGO_PKG_VERSION");
    match fetch_latest() {
        Ok(info) => {
            if is_newer_than(current, &info.version) {
                UpdateOutcome::Available(Box::new(info))
            } else {
                UpdateOutcome::UpToDate(info.version)
            }
        }
        Err(reason) => UpdateOutcome::Failed(reason),
    }
}

/// Only accept asset URLs that genuinely belong to this repository's
/// release downloads. The URL comes from an API response, so treat it as
/// untrusted input and refuse anything unexpected.
fn asset_url_is_trusted(url: &str) -> bool {
    url.starts_with(&format!(
        "https://github.com/{GITHUB_REPO}/releases/download/"
    )) || url.starts_with("https://objects.githubusercontent.com/")
}

/// Download the `.deb` to the system temp dir and confirm it is a real
/// Debian package before it is ever handed to the installer.
pub fn download_deb(asset_url: &str, version: &str) -> Result<std::path::PathBuf, String> {
    if !asset_url_is_trusted(asset_url) {
        return Err("asset URL is not from this project's releases".to_string());
    }
    let dest = std::env::temp_dir().join(format!("rmux-update-{version}.deb"));
    let dest_arg = dest.to_string_lossy().to_string();
    let status = std::process::Command::new("curl")
        .args(["-sL", "--max-time", "180", "-o", &dest_arg, asset_url])
        .status()
        .map_err(|e| format!("could not run curl: {e}"))?;
    if !status.success() {
        return Err("download failed".to_string());
    }
    let info = std::process::Command::new("dpkg-deb")
        .args(["--info", &dest_arg])
        .output()
        .map_err(|_| "could not verify the downloaded file".to_string())?;
    if !info.status.success() {
        let _ = std::fs::remove_file(&dest);
        return Err("downloaded file is not a valid installer".to_string());
    }
    Ok(dest)
}

/// Install a verified `.deb`. `pkexec` always prompts for administrator
/// authentication, so this cannot run silently.
pub fn install_deb(path: &std::path::Path) -> Result<(), String> {
    let status = std::process::Command::new("pkexec")
        .args(["dpkg", "-i", &path.to_string_lossy().to_string()])
        .status()
        .map_err(|e| format!("could not run installer: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("installation was cancelled or failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_patch_detected() {
        assert!(is_newer_than("1.0.1", "1.0.2"));
    }

    #[test]
    fn same_version_is_not_newer() {
        assert!(!is_newer_than("1.0.1", "v1.0.1"));
    }

    #[test]
    fn older_major_is_not_newer() {
        assert!(!is_newer_than("2.0.0", "1.9.9"));
    }

    #[test]
    fn untrusted_asset_url_rejected() {
        assert!(!asset_url_is_trusted("https://example.com/rmux_9.9.9_amd64.deb"));
        assert!(asset_url_is_trusted(
            "https://github.com/geneshairzan/rmux/releases/download/v1.0.2/rmux_1.0.2_amd64.deb"
        ));
    }
}
