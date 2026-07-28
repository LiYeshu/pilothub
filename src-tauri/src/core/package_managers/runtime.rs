use std::fs;
use std::io::{Cursor, Write};
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::apm::ApmAdapter;
use crate::core::network_proxy::github_http_client;

const APM_LATEST_RELEASE_URL: &str = "https://api.github.com/repos/microsoft/apm/releases/latest";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedApmRuntime {
    pub version: String,
    pub binary_path: PathBuf,
}

impl ManagedApmRuntime {
    pub fn adapter(&self) -> ApmAdapter {
        ApmAdapter::new(self.binary_path.to_string_lossy())
    }
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub fn managed_apm_root(home: &Path) -> PathBuf {
    home.join(".pilothub/package-managers/apm")
}

pub fn find_managed_apm(root: &Path) -> Result<Option<ManagedApmRuntime>> {
    let current_path = root.join("current");
    let version = match fs::read_to_string(&current_path) {
        Ok(version) => version.trim().to_string(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("read APM runtime marker {current_path:?}"))
        }
    };
    validate_version(&version)?;
    find_runtime(&version, &root.join(&version)).map(Some)
}

pub fn install_latest_apm(proxy_url: &str, root: &Path) -> Result<ManagedApmRuntime> {
    let client = github_http_client(proxy_url, Some(60))?;
    let release = fetch_latest_release(&client)?;
    let archive_name = apm_asset_name(std::env::consts::OS, std::env::consts::ARCH)?;
    let checksum_name = format!("{archive_name}.sha256");
    let archive_url = release_asset_url(&release, archive_name)?;
    let checksum_url = release_asset_url(&release, &checksum_name)?;
    let archive = download_bytes(&client, archive_url)?;
    let checksum = download_bytes(&client, checksum_url)?;

    install_apm_archive(
        &release.tag_name,
        archive_name,
        &archive,
        std::str::from_utf8(&checksum).context("APM checksum is not UTF-8")?,
        root,
    )
}

fn fetch_latest_release(client: &Client) -> Result<GithubRelease> {
    client
        .get(APM_LATEST_RELEASE_URL)
        .header("User-Agent", "pilothub")
        .header("Accept", "application/vnd.github+json")
        .send()
        .context("request latest Microsoft APM release")?
        .error_for_status()
        .context("latest Microsoft APM release request failed")?
        .json()
        .context("parse latest Microsoft APM release")
}

fn release_asset_url<'a>(release: &'a GithubRelease, name: &str) -> Result<&'a str> {
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .map(|asset| asset.browser_download_url.as_str())
        .with_context(|| format!("Microsoft APM release is missing asset {name}"))
}

fn download_bytes(client: &Client, url: &str) -> Result<Vec<u8>> {
    Ok(client
        .get(url)
        .header("User-Agent", "pilothub")
        .send()
        .with_context(|| format!("download Microsoft APM asset {url}"))?
        .error_for_status()
        .with_context(|| format!("Microsoft APM asset request failed: {url}"))?
        .bytes()
        .with_context(|| format!("read Microsoft APM asset {url}"))?
        .to_vec())
}

fn apm_asset_name(os: &str, arch: &str) -> Result<&'static str> {
    match (os, arch) {
        ("macos", "aarch64") => Ok("apm-darwin-arm64.tar.gz"),
        ("macos", "x86_64") => Ok("apm-darwin-x86_64.tar.gz"),
        ("linux", "aarch64") => Ok("apm-linux-arm64.tar.gz"),
        ("linux", "x86_64") => Ok("apm-linux-x86_64.tar.gz"),
        ("windows", "x86_64") => Ok("apm-windows-x86_64.zip"),
        _ => bail!("Microsoft APM does not publish an asset for {os}/{arch}"),
    }
}

fn install_apm_archive(
    version: &str,
    archive_name: &str,
    archive: &[u8],
    checksum: &str,
    root: &Path,
) -> Result<ManagedApmRuntime> {
    validate_version(version)?;
    verify_checksum(archive, checksum)?;
    fs::create_dir_all(root).with_context(|| format!("create APM runtime root {root:?}"))?;

    let install_dir = root.join(version);
    if install_dir.exists() {
        let runtime = find_runtime(version, &install_dir)?;
        write_current_version(root, version)?;
        return Ok(runtime);
    }

    let staging = root.join(format!(".staging-{}", Uuid::new_v4()));
    fs::create_dir(&staging)
        .with_context(|| format!("create APM staging directory {staging:?}"))?;
    let result: Result<ManagedApmRuntime> = (|| {
        extract_archive(archive_name, archive, &staging)?;
        let staged = find_runtime(version, &staging)?;
        let relative_binary = staged
            .binary_path
            .strip_prefix(&staging)
            .context("APM binary escaped staging directory")?
            .to_path_buf();
        fs::rename(&staging, &install_dir)
            .with_context(|| format!("activate APM runtime {install_dir:?}"))?;
        Ok(ManagedApmRuntime {
            version: version.to_string(),
            binary_path: install_dir.join(relative_binary),
        })
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    let runtime = result?;
    write_current_version(root, version)?;
    Ok(runtime)
}

fn write_current_version(root: &Path, version: &str) -> Result<()> {
    let marker = root.join("current");
    let staging = root.join(format!(".current-{}", Uuid::new_v4()));
    fs::write(&staging, version)
        .with_context(|| format!("write APM runtime marker {staging:?}"))?;
    if marker.exists() {
        fs::remove_file(&marker)
            .with_context(|| format!("replace APM runtime marker {marker:?}"))?;
    }
    fs::rename(&staging, &marker).with_context(|| format!("activate APM runtime marker {marker:?}"))
}

fn validate_version(version: &str) -> Result<()> {
    if version.is_empty()
        || Path::new(version)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("invalid Microsoft APM release version");
    }
    Ok(())
}

fn verify_checksum(bytes: &[u8], checksum: &str) -> Result<()> {
    let expected = checksum
        .split_whitespace()
        .next()
        .context("Microsoft APM checksum file is empty")?;
    let actual = hex::encode(Sha256::digest(bytes));
    if !actual.eq_ignore_ascii_case(expected) {
        bail!("Microsoft APM SHA-256 checksum mismatch");
    }
    Ok(())
}

fn extract_archive(name: &str, bytes: &[u8], destination: &Path) -> Result<()> {
    if name.ends_with(".tar.gz") {
        extract_tar_gz(bytes, destination)
    } else if name.ends_with(".zip") {
        extract_zip(bytes, destination)
    } else {
        bail!("unsupported Microsoft APM archive format: {name}")
    }
}

fn extract_tar_gz(bytes: &[u8], destination: &Path) -> Result<()> {
    let decoder = GzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(decoder);
    for entry in archive
        .entries()
        .context("read Microsoft APM tar archive")?
    {
        let mut entry = entry.context("read Microsoft APM tar entry")?;
        let relative = safe_relative_path(&entry.path().context("read tar entry path")?)?;
        let target = destination.join(relative);
        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("create APM archive directory {target:?}"))?;
        } else if entry_type.is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create APM archive parent {parent:?}"))?;
            }
            entry
                .unpack(&target)
                .with_context(|| format!("extract APM archive file {target:?}"))?;
        } else {
            bail!("Microsoft APM archive contains an unsupported entry type");
        }
    }
    Ok(())
}

fn extract_zip(bytes: &[u8], destination: &Path) -> Result<()> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(bytes)).context("read Microsoft APM zip archive")?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .context("read Microsoft APM zip entry")?;
        let relative = entry
            .enclosed_name()
            .context("Microsoft APM zip entry escaped destination")?
            .to_path_buf();
        let target = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("create APM zip directory {target:?}"))?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create APM zip parent {parent:?}"))?;
        }
        let mut file =
            fs::File::create(&target).with_context(|| format!("create APM file {target:?}"))?;
        std::io::copy(&mut entry, &mut file)
            .with_context(|| format!("extract APM zip file {target:?}"))?;
        file.flush()
            .with_context(|| format!("flush APM zip file {target:?}"))?;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&target, fs::Permissions::from_mode(mode))
                .with_context(|| format!("set APM file permissions {target:?}"))?;
        }
    }
    Ok(())
}

fn safe_relative_path(path: &Path) -> Result<PathBuf> {
    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        bail!("Microsoft APM archive entry escaped destination");
    }
    Ok(path.to_path_buf())
}

fn find_runtime(version: &str, root: &Path) -> Result<ManagedApmRuntime> {
    let binary_name = if cfg!(windows) { "apm.exe" } else { "apm" };
    let binary_path = walkdir::WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| entry.file_type().is_file() && entry.file_name() == binary_name)
        .map(|entry| entry.into_path())
        .with_context(|| format!("Microsoft APM archive does not contain {binary_name}"))?;
    Ok(ManagedApmRuntime {
        version: version.to_string(),
        binary_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_supported_release_assets() {
        assert_eq!(
            apm_asset_name("macos", "aarch64").unwrap(),
            "apm-darwin-arm64.tar.gz"
        );
        assert_eq!(
            apm_asset_name("windows", "x86_64").unwrap(),
            "apm-windows-x86_64.zip"
        );
        assert!(apm_asset_name("windows", "aarch64").is_err());
    }

    #[test]
    fn rejects_checksum_mismatch() {
        let error = verify_checksum(b"archive", "deadbeef archive.tar.gz").unwrap_err();
        assert!(error.to_string().contains("checksum mismatch"));
    }

    #[test]
    fn rejects_archive_path_traversal() {
        assert!(safe_relative_path(Path::new("../apm")).is_err());
        assert!(safe_relative_path(Path::new("/tmp/apm")).is_err());
    }

    #[test]
    fn installs_verified_tar_archive_atomically() {
        let archive = test_tar_archive("apm-darwin-arm64/apm", b"binary");
        let checksum = hex::encode(Sha256::digest(&archive));
        let root = tempfile::tempdir().unwrap();

        let runtime =
            install_apm_archive("v1.2.3", "apm.tar.gz", &archive, &checksum, root.path()).unwrap();

        assert_eq!(runtime.version, "v1.2.3");
        assert_eq!(fs::read(&runtime.binary_path).unwrap(), b"binary");
        assert_eq!(
            fs::read_to_string(root.path().join("current")).unwrap(),
            "v1.2.3"
        );
        assert_eq!(find_managed_apm(root.path()).unwrap(), Some(runtime));
        assert!(!root.path().read_dir().unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".staging-")));
    }

    #[test]
    fn returns_none_without_active_runtime_marker() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(find_managed_apm(root.path()).unwrap(), None);
    }

    #[test]
    fn extracts_safe_zip_archive() {
        let destination = tempfile::tempdir().unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer
            .start_file(
                "apm-windows-x86_64/apm.exe",
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
        writer.write_all(b"binary").unwrap();
        let bytes = writer.finish().unwrap().into_inner();

        extract_zip(&bytes, destination.path()).unwrap();

        assert_eq!(
            fs::read(destination.path().join("apm-windows-x86_64/apm.exe")).unwrap(),
            b"binary"
        );
    }

    fn test_tar_archive(path: &str, contents: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let encoder = flate2::write::GzEncoder::new(&mut bytes, flate2::Compression::default());
            let mut archive = tar::Builder::new(encoder);
            let mut header = tar::Header::new_gnu();
            header.set_size(contents.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            archive
                .append_data(&mut header, path, Cursor::new(contents))
                .unwrap();
            archive.into_inner().unwrap().finish().unwrap();
        }
        bytes
    }
}
