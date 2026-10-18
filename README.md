<p align="center">
  <img src="assets/banners/ezcmd_banner.png" alt="ezCMD Banner" width="100%">
</p>

<div align="center">

  <img src="assets/logos/ezcmd_logo.png" alt="ezCMD Logo" width="110" height="110">

  <h1>ezCMD</h1>
  <p><b>Type what you want to do. Get the commands, in order.</b><br>
  <i>A fast, offline, OS-aware terminal cheatsheet that remembers so you don't have to.</i></p>

  <p>
    <a href="https://github.com/ItzEvhyx/ezCMD/releases"><img src="https://img.shields.io/github/v/release/ItzEvhyx/ezCMD?color=00d2ff&style=for-the-badge&logo=github" alt="Release"></a>
    <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-7928CA?style=for-the-badge&logo=linux" alt="Platform">
    <img src="https://img.shields.io/badge/Startup-%3C%2020ms-00e676?style=for-the-badge&logo=speedtest" alt="Startup">
    <img src="https://img.shields.io/badge/Size-%3C%201.2MB-00b0ff?style=for-the-badge" alt="Size">
    <img src="https://img.shields.io/badge/Rust-1.70%2B-dea584?style=for-the-badge&logo=rust" alt="Rust">
  </p>

</div>

<p align="center">
  <img src="assets/banners/ezcmd_ascii.png" alt="ezCMD ASCII Art" width="600">
</p>

<p align="center">
  <a href="#preview"><b>Preview</b></a> &nbsp;|&nbsp;
  <a href="#features"><b>Features</b></a> &nbsp;|&nbsp;
  <a href="#platforms"><b>Platforms</b></a> &nbsp;|&nbsp;
  <a href="#updating"><b>Updating</b></a> &nbsp;|&nbsp;
  <a href="#installation"><b>Installation</b></a> &nbsp;|&nbsp;
  <a href="#usage"><b>Usage</b></a> &nbsp;|&nbsp;
  <a href="#uninstalling"><b>Uninstalling</b></a> &nbsp;|&nbsp;
  <a href="#releases"><b>Releases</b></a> &nbsp;|&nbsp;
  <a href="#author"><b>Author</b></a> &nbsp;|&nbsp;
  <a href="#license"><b>License</b></a>
</p>

---

<a id="preview"></a>

## <img src="https://api.iconify.design/lucide/terminal.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Preview

```text
$ ezcmd initializing repo

 ___[ Initializing a Repo ]_________________________________________________________| 
|                                                                                   |
|  git init                          --- Initializes a repo in the current folder   |
|  git add .                         --- Stages all new and changed files           |
|  git commit -m "Initial commit"    --- Saves the staged files as the first commit |
|  git branch -M main                --- Renames the current branch to main         |
|  git remote add origin <repo-url>  --- Links the repo to a remote on GitHub       |
|  git push -u origin main           --- Uploads main and sets it as the upstream   |
|___________________________________________________________________________________|
```

> [!NOTE]
> **Aesthetic Terminal Box:** In modern terminals, borders and notes are styled with subtle dimming, commands are rendered in **bold `#96d0fb`**, and `<placeholders>` are highlighted in **yellow**. Color automatically switches off in pipes or when `NO_COLOR` is detected.

---

<a id="features"></a>

## <img src="https://api.iconify.design/lucide/layers.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Features

| | Feature | Description |
|:---:|:---|:---|
| <img src="https://api.iconify.design/lucide/target.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Task-Oriented Search** | Search for what you intend to accomplish (`initializing repo`, `undo last commit`, `delete files`) rather than trying to remember syntax. |
| <img src="https://api.iconify.design/lucide/search.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Intelligent Fuzzy Matcher** | Handles typos, transposed words, and natural phrasing using custom token-stemming and Damerau-Levenshtein distance. |
| <img src="https://api.iconify.design/lucide/library.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Browsable Command Library** | Browse categories, tool tables of contents, and focused sections without dumping every command at once. |
| <img src="https://api.iconify.design/lucide/tags.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Tool Aliases** | Reach tools by their short flag or a non-shadowed alternate name, such as `-rebase` for Git. |
| <img src="https://api.iconify.design/lucide/monitor.svg?color=%2300b0ff" width="22" height="22" alt=""> | **OS-Aware Resolution** | Automatically detects whether you are on Linux, macOS, or Windows CMD and presents commands that work for your operating system. |
| <img src="https://api.iconify.design/lucide/package.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Offline by Default** | Cheatsheet data is compiled directly into the binary. Lookups make zero network requests, collect no telemetry, and run no background daemons. |
| <img src="https://api.iconify.design/lucide/refresh-cw.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Verified Self-Update** | `ezcmd update` fetches only a user-requested GitHub Release, verifies its SHA-256 file, and replaces the installed global binary. |
| <img src="https://api.iconify.design/lucide/zap.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Fast Search** | Launches, matches, formats, and terminates in **under 20 milliseconds** with less than 1.2 MB footprint. |
| <img src="https://api.iconify.design/lucide/square.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Fully Enclosed Box Frame** | Retro-modern ASCII borders enclose every output neatly on all sides. |
| <img src="https://api.iconify.design/lucide/flag.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Welcome Banner & Quickstart** | Beautiful gradient ASCII art greeting on first install with a built-in how-to guide. |
| <img src="https://api.iconify.design/lucide/trash-2.svg?color=%2300b0ff" width="22" height="22" alt=""> | **Built-in Uninstaller** | Clean removal with `ezcmd --uninstall`, no leftover files. |

---

<a id="platforms"></a>

## <img src="https://api.iconify.design/lucide/monitor.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Supported Platforms

| Operating System | Architecture | Status |
|:---|:---|:---:|
| **Linux** | x86_64, aarch64 (ARM64) | Supported |
| **macOS** | Apple Silicon (M1/M2/M3), Intel x86_64 | Supported |
| **Windows** | Windows 10, 11 (x86_64 CMD & PowerShell) | Supported |

---

<a id="updating"></a>

## <img src="https://api.iconify.design/lucide/refresh-cw.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Updating

If you already have **ezcmd 2.1.0 or later**, future releases do not require uninstalling or rerunning the installer. Update the global binary with:

```bash
ezcmd --update
```

`ezcmd update` works too. The updater downloads the matching GitHub Release for your OS and CPU, verifies its published SHA-256 checksum, and replaces the installed executable. Regular searches remain offline.

If you are on v1 or v2.0.0, run the installer below once to get v2.1.1. After that, use `ezcmd --update` for later releases.

---

<a id="installation"></a>

## <img src="https://api.iconify.design/lucide/download.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Installation

<details open>
<summary><b>Quick Automated Install (Recommended)</b></summary>
<br>

#### Linux & macOS (Bash / Zsh / Fish)

Run the one-liner install script. It auto-detects your OS and architecture:

```bash
curl -fsSL https://raw.githubusercontent.com/ItzEvhyx/ezCMD/main/install.sh | bash
```

**What this does:**
1. Downloads the correct prebuilt binary from the [latest GitHub release](https://github.com/ItzEvhyx/ezCMD/releases)
2. Copies it to `~/.local/bin/ezcmd`
3. Prints the welcome banner with a quickstart guide

> [!NOTE]
> If `~/.local/bin` is not in your `PATH`, the installer will print instructions on how to add it for your shell.

#### Windows (PowerShell)

Open PowerShell and run:

```powershell
irm https://raw.githubusercontent.com/ItzEvhyx/ezCMD/main/install.ps1 | iex
```

> [!IMPORTANT]
> **Windows security notice:** The Windows build of ezCMD is not yet code-signed. Windows 11 **Smart App Control**, or a work/school device policy, may block `ezcmd.exe` the first time you run it with the message *"An Application Control policy has blocked this file."* If that happens, see [Windows troubleshooting](#windows-troubleshooting) below.

<a id="windows-troubleshooting"></a>

<details>
<summary><b>Windows troubleshooting: common errors and fixes</b></summary>
<br>

**`An Application Control policy has blocked this file`** (shown when you run `ezcmd`)

- **Cause:** Smart App Control blocks apps that are unsigned and have no reputation yet. There is no per-app exception for it. A managed work or school device can show the same message through its own policy.
- **Fix:** Open **Windows Security**, go to **App & browser control**, then **Smart App Control settings**, and choose **Off**. Turning it off lowers protection against unknown programs, so only do this if you are comfortable with that. On older Windows 11 builds, turning it off cannot be undone without reinstalling Windows. Recent Windows 11 updates allow switching it back on later.
- **Managed device:** If Smart App Control is already off, or the PC belongs to a workplace or school, ask the device administrator to allow `%USERPROFILE%\.local\bin\ezcmd.exe`.

**`Cannot bind argument to parameter 'Path' because it is null`** (shown while installing)

- **Cause:** An older version of the installer could not find its own location when piped into PowerShell. This is fixed in the current `install.ps1`.
- **Fix:** Run the install command again. If the error persists, GitHub may still be serving a cached copy, so wait a few minutes and retry. You can also run the installer as a file instead:

```powershell
irm https://raw.githubusercontent.com/ItzEvhyx/ezCMD/main/install.ps1 -OutFile install.ps1
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

**`ezcmd is not recognized as the name of a cmdlet`** (or `not recognized as an internal or external command`)

- **Cause:** The terminal you are using was opened before the install, so it has not picked up the new `PATH`.
- **Fix:** Close the terminal and open a new one. If it still fails, add `%USERPROFILE%\.local\bin` to your `PATH` using the steps in [Terminal-Specific PATH Setup](#installation) below.

**`running scripts is disabled on this system`** (when running `install.ps1` as a file)

- **Cause:** PowerShell's execution policy blocks local script files.
- **Fix:** Run the script with a one-time bypass, which does not change your system settings:

```powershell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

**Download fails or returns a 404**

- **Cause:** A network or proxy problem, a GitHub API rate limit, or a release that is missing its Windows file.
- **Fix:** Retry after a moment. To install manually, download `ezcmd-<version>-x86_64-pc-windows-msvc.zip` from the [releases page](https://github.com/ItzEvhyx/ezCMD/releases), extract `ezcmd.exe`, and place it in `%USERPROFILE%\.local\bin`.

</details>

</details>

<details>
<summary><b>From Source (Rust / Cargo)</b></summary>
<br>

Requires [Rust 1.70+](https://rustup.rs/).

```bash
git clone https://github.com/ItzEvhyx/ezCMD.git
cd ezCMD
cargo build --release

# Run the built-in installer (copies to ~/.local/bin + shows welcome)
./target/release/ezcmd --install
```

</details>

<details>
<summary><b>Terminal-Specific PATH Setup</b></summary>
<br>

If `~/.local/bin` is not in your `PATH` after installation, add it for your shell:

#### Fish
```fish
set -Ua fish_user_paths ~/.local/bin
```

#### Bash
```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc
```

#### Zsh
```zsh
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
```

#### Windows CMD
```cmd
setx PATH "%PATH%;%USERPROFILE%\.local\bin"
```

</details>

### Verify installation

```bash
ezcmd --version
# → ezcmd 2.1.1
```

---

<a id="usage"></a>

## <img src="https://api.iconify.design/lucide/book-open.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Usage & Cheatsheet

| Command | Description |
|:---|:---|
| `ezcmd <query>` | Find a workflow or command |
| `ezcmd --help` | Browse OS-filtered categories and tools |
| `ezcmd --help -<tool>` | Show a tool's groups and section titles (e.g. `-git`, `-docker`) |
| `ezcmd --help -<tool> -<section>` | Show the matching section's commands |
| `ezcmd --help -<section>` | Search matching sections across every tool |
| `ezcmd --help -<os>` | Browse as Linux, macOS, or Windows |
| `ezcmd --shell <cmd\|powershell> <query>` | Choose Windows command syntax explicitly |
| `ezcmd --plain <query>` | Raw plain output (no ANSI, no box frames) |
| `ezcmd --install` | Install ezcmd to PATH + show welcome |
| `ezcmd update` / `ezcmd --update` | Download and install the latest published release |
| `ezcmd --welcome` | Display ASCII welcome banner & quickstart |
| `ezcmd --uninstall` | Uninstall ezcmd completely from system |
| `ezcmd --version` | Print installed ezcmd version |

### Examples

```bash
# Everyday workflows
ezcmd initializing repo
ezcmd undo last commit
ezcmd how to branch
ezcmd stash changes
ezcmd delete folder

# Search by exact command syntax
ezcmd -- reset --hard

# Browse the command library
ezcmd --help
ezcmd --help -docker
ezcmd --help -docker -logs
ezcmd --help -initialize

# Tool aliases and OS-specific exploration
ezcmd --help -rebase
ezcmd --help -windows -winget

# Use PowerShell syntax when browsing Windows commands (cmd is also available)
ezcmd --help -windows -shell -remove folder --shell powershell
```

### Tool aliases

Each tool has a canonical flag such as `-docker` and may have alternate names shown in its `Also matches` line. Non-shadowed aliases can be used as tool flags, so `ezcmd --help -rebase` opens Git. If an alias is also meaningful as a section search, select the tool first: `ezcmd --help -git -worktree`.

### Handling Ambiguous Queries

When multiple sections match your query equally well, ezCMD provides an interactive numbered prompt:

```text
$ ezcmd undo

Multiple matches for "undo":
  1. Undo & Rollback
  2. Undoing the Last Commit
  3. Restoring a Deleted File
Choose [1/2/3]: 2
```

*(In scripts or non-interactive pipes, ezCMD automatically selects the top match without blocking).*

---

<a id="uninstalling"></a>

## <img src="https://api.iconify.design/lucide/trash-2.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Uninstalling

Uninstall ezCMD cleanly with the built-in command:

```bash
ezcmd --uninstall
```

You can also find it through fuzzy search:

```bash
ezcmd uninstall
ezcmd remove tool
```

For non-interactive environments (CI, scripts):

```bash
ezcmd --uninstall -y
```

This removes the binary from `~/.local/bin`, symlinks, and any configuration files. If ezcmd is not installed, it will let you know with a clear error message.

---

<a id="releases"></a>

## <img src="https://api.iconify.design/lucide/refresh-cw.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Release Automation

Every tagged push (`v*`) triggers a [GitHub Actions workflow](.github/workflows/release.yml) that:

1. Cross-compiles for **5 targets** (Linux x86/ARM, macOS x86/ARM, Windows)
2. Packages as `.tar.gz` (Unix) or `.zip` (Windows)
3. Generates SHA-256 checksums
4. Creates a GitHub Release with all artifacts

Published release archives are also the update channel for `ezcmd update`. Tag a new semantic version such as `v2.1.1`, let the workflow finish, then existing 2.1.0+ installations can update themselves globally with that command.

---

<a id="author"></a>

## <img src="https://api.iconify.design/lucide/user.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; Author & Credits

<p align="center">
  <img src="assets/logos/ezcmd_logo_typography.png" alt="ezCMD Typography" width="300">
</p>

Created by **ItzEvhyx**

- **GitHub:** [@ItzEvhyx](https://github.com/ItzEvhyx)
- **Repository:** [https://github.com/ItzEvhyx/ezCMD](https://github.com/ItzEvhyx/ezCMD)

---

<a id="license"></a>

## <img src="https://api.iconify.design/lucide/file-text.svg?color=%2300b0ff" width="26" height="26" align="middle" alt="">&nbsp; License

MIT License. See [LICENSE](LICENSE) for details.
