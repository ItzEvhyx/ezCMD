//! Explicit, checksum-verified self-update support.
//!
//! Normal ezcmd lookups remain fully offline. This module is reached only by
//! `ezcmd update` or `ezcmd --update`, where a user has asked to contact the
//! release service and replace their installed executable.

use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const REPOSITORY: &str = "ItzEvhyx/ezCMD";
const API_URL: &str = "https://api.github.com/repos/ItzEvhyx/ezCMD/releases/latest";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Version {
    major: u64,
    minor: u64,
    patch: u64,
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Version {
    fn parse(value: &str) -> Option<Self> {
        let value = value.trim().strip_prefix('v').unwrap_or(value.trim());
        let mut parts = value.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }
}

fn update_error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

fn command_output(program: &str, args: &[&str]) -> io::Result<Vec<u8>> {
    let output = Command::new(program).args(args).output().map_err(|error| {
        update_error(format!(
            "could not run {program}; install it and try again: {error}"
        ))
    })?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        let detail = String::from_utf8_lossy(&output.stderr);
        Err(update_error(format!("{program} failed: {}", detail.trim())))
    }
}

fn latest_release_tag(body: &str) -> Option<String> {
    let after_key = body.split_once("\"tag_name\"")?.1;
    let after_colon = after_key.split_once(':')?.1.trim_start();
    let value = after_colon.strip_prefix('"')?;
    let end = value.find('"')?;
    Some(value[..end].to_owned())
}

fn latest_tag() -> io::Result<String> {
    let body = command_output(
        "curl",
        &[
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--header",
            "User-Agent: ezcmd-updater",
            API_URL,
        ],
    )?;
    let body =
        String::from_utf8(body).map_err(|_| update_error("release API returned non-UTF-8 data"))?;
    latest_release_tag(&body).ok_or_else(|| update_error("release API response has no tag_name"))
}

fn target() -> io::Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok("x86_64-unknown-linux-musl"),
        ("linux", "aarch64") => Ok("aarch64-unknown-linux-musl"),
        ("macos", "x86_64") => Ok("x86_64-apple-darwin"),
        ("macos", "aarch64") => Ok("aarch64-apple-darwin"),
        ("windows", "x86_64") => Ok("x86_64-pc-windows-msvc"),
        (os, arch) => Err(update_error(format!(
            "no published ezcmd updater target for {os}/{arch}"
        ))),
    }
}

fn archive_extension() -> &'static str {
    if cfg!(windows) {
        "zip"
    } else {
        "tar.gz"
    }
}

fn release_url(tag: &str, asset: &str) -> String {
    format!("https://github.com/{REPOSITORY}/releases/download/{tag}/{asset}")
}

fn download(url: &str, destination: &Path) -> io::Result<()> {
    let destination = destination.to_string_lossy();
    command_output(
        "curl",
        &[
            "--fail",
            "--location",
            "--silent",
            "--show-error",
            "--retry",
            "2",
            "--connect-timeout",
            "10",
            "--output",
            &destination,
            url,
        ],
    )?;
    Ok(())
}

fn expected_checksum(checksum_file: &Path) -> io::Result<String> {
    let contents = fs::read_to_string(checksum_file)?;
    let checksum = contents
        .split_whitespace()
        .next()
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| update_error("release checksum file is malformed"))?;
    Ok(checksum.to_ascii_lowercase())
}

fn checksum(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 32 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn unique_temp_dir() -> io::Result<PathBuf> {
    let path = std::env::temp_dir().join(format!(
        "ezcmd-update-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| update_error("system clock is before the Unix epoch"))?
            .as_nanos()
    ));
    fs::create_dir(&path)?;
    Ok(path)
}

fn extract(archive: &Path, output: &Path) -> io::Result<PathBuf> {
    #[cfg(not(windows))]
    {
        let archive_arg = archive.to_string_lossy();
        let output_arg = output.to_string_lossy();
        command_output("tar", &["-xzf", &archive_arg, "-C", &output_arg])?;
        let binary = output.join("ezcmd");
        if binary.is_file() {
            return Ok(binary);
        }
    }

    #[cfg(windows)]
    {
        let script = format!(
            "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
            powershell_literal(archive),
            powershell_literal(output)
        );
        command_output(
            "powershell",
            &["-NoProfile", "-NonInteractive", "-Command", &script],
        )?;
        let binary = output.join("ezcmd.exe");
        if binary.is_file() {
            return Ok(binary);
        }
    }

    Err(update_error(
        "release archive did not contain the ezcmd executable",
    ))
}

#[cfg(windows)]
fn powershell_literal(path: &Path) -> String {
    path.to_string_lossy().replace('\'', "''")
}

fn installed_executable() -> io::Result<PathBuf> {
    let executable = std::env::current_exe()?;
    if executable
        .components()
        .any(|component| component.as_os_str() == "target")
    {
        return Err(update_error(
            "refusing to update a development build; install ezcmd first",
        ));
    }
    Ok(executable)
}

#[cfg(unix)]
fn replace_executable(binary: &Path, installed: &Path, _temp_dir: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let parent = installed
        .parent()
        .ok_or_else(|| update_error("installed executable has no parent directory"))?;
    let staged = parent.join(format!(".ezcmd-update-{}", std::process::id()));
    fs::copy(binary, &staged)?;
    let mut permissions = fs::metadata(&staged)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&staged, permissions)?;
    fs::rename(&staged, installed)?;
    Ok(())
}

#[cfg(windows)]
fn replace_executable(binary: &Path, installed: &Path, temp_dir: &Path) -> io::Result<()> {
    let parent = installed
        .parent()
        .ok_or_else(|| update_error("installed executable has no parent directory"))?;
    let staged = parent.join(format!("ezcmd.exe.update-{}", std::process::id()));
    fs::copy(binary, &staged)?;

    let script_path = temp_dir.join("replace-ezcmd.cmd");
    let script = format!(
        "@echo off\r\nping 127.0.0.1 -n 2 > nul\r\nmove /Y \"{}\" \"{}\" > nul\r\ndel \"%~f0\"\r\n",
        staged.display(),
        installed.display()
    );
    fs::write(&script_path, script)?;
    Command::new("cmd")
        .args(["/C", &script_path.to_string_lossy()])
        .spawn()
        .map_err(|error| {
            update_error(format!(
                "could not start Windows replacement helper: {error}"
            ))
        })?;
    Ok(())
}

/// Download the latest published release for this OS/architecture and replace
/// the installed executable. No network call happens outside this explicit path.
pub fn run<W: Write>(w: &mut W) -> io::Result<i32> {
    writeln!(w, "Checking for ezcmd updates...")?;
    // Refuse before any network request when invoked from `target/`; a release
    // build in the repository is not a global installation to replace.
    let installed = installed_executable()?;
    let tag = latest_tag()?;
    let latest = Version::parse(&tag).ok_or_else(|| {
        update_error(format!(
            "latest release tag {tag:?} is not vMAJOR.MINOR.PATCH"
        ))
    })?;
    let current_text = env!("CARGO_PKG_VERSION");
    let current = Version::parse(current_text).expect("package version must be semantic version");

    if latest <= current {
        writeln!(w, "ezcmd {current_text} is already up to date.")?;
        return Ok(0);
    }

    let target = target()?;
    let asset = format!("ezcmd-{tag}-{target}.{}", archive_extension());
    let temp_dir = unique_temp_dir()?;
    let archive = temp_dir.join(&asset);
    let checksum_file = temp_dir.join(format!("{asset}.sha256"));

    let result = (|| {
        writeln!(w, "Downloading ezcmd {tag} for {target}...")?;
        download(&release_url(&tag, &asset), &archive)?;
        download(
            &release_url(&tag, &format!("{asset}.sha256")),
            &checksum_file,
        )?;
        let expected = expected_checksum(&checksum_file)?;
        let actual = checksum(&archive)?;
        if actual != expected {
            return Err(update_error(
                "downloaded archive failed SHA-256 verification",
            ));
        }
        let binary = extract(&archive, &temp_dir)?;
        replace_executable(&binary, &installed, &temp_dir)?;
        Ok(())
    })();

    #[cfg(not(windows))]
    let _ = fs::remove_dir_all(&temp_dir);

    result?;
    #[cfg(windows)]
    writeln!(
        w,
        "ezcmd {tag} will replace the current executable after this command exits."
    )?;
    #[cfg(not(windows))]
    writeln!(w, "Updated ezcmd {current_text} to {tag}.")?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_release_tags_and_orders_versions() {
        assert_eq!(Version::parse("v2.1.0"), Version::parse("2.1.0"));
        assert!(Version::parse("2.1.0") > Version::parse("2.0.9"));
        assert!(Version::parse("v2.1").is_none());
        assert!(Version::parse("v2.1.0-beta").is_none());
    }

    #[test]
    fn extracts_tag_name_from_release_json() {
        let body = r#"{"url":"https://api.github.com/releases/1","tag_name":"v2.1.0"}"#;
        assert_eq!(latest_release_tag(body).as_deref(), Some("v2.1.0"));
    }

    #[test]
    fn release_asset_names_follow_the_workflow_contract() {
        assert_eq!(
            release_url("v2.1.0", "ezcmd-v2.1.0-x86_64-unknown-linux-musl.tar.gz"),
            "https://github.com/ItzEvhyx/ezCMD/releases/download/v2.1.0/ezcmd-v2.1.0-x86_64-unknown-linux-musl.tar.gz"
        );
    }
}
