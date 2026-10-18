use ezcmd::content::all_tools;
use ezcmd::model::{Command, Section};
use ezcmd::os::Os;
use ezcmd::render::{render_section, RenderCtx};

fn get_section<'a>(tool_name: &str, title: &str) -> &'a Section {
    let tool = all_tools()
        .find(|tool| tool.slug == tool_name)
        .unwrap_or_else(|| panic!("tool {tool_name} not found"));
    tool.sections()
        .find(|s| s.title == title)
        .unwrap_or_else(|| panic!("section {title} not found"))
}

#[test]
fn test_snapshot_wide_layout() {
    let sec = get_section("git", "Initializing a Repo");
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 100,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("wide_layout_no_color", output);
}

#[test]
fn test_snapshot_wide_layout_colored() {
    let sec = get_section("git", "Initializing a Repo");
    let ctx = RenderCtx {
        color: true,
        plain: false,
        width: 100,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("wide_layout_colored", output);
}

#[test]
fn test_snapshot_narrow_layout() {
    let sec = get_section("git", "Initializing a Repo");
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 60,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("narrow_layout", output);
}

#[test]
fn test_snapshot_plain_mode() {
    let sec = get_section("git", "Initializing a Repo");
    let ctx = RenderCtx {
        color: false,
        plain: true,
        width: 80,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("plain_mode", output);
}

#[test]
fn test_snapshot_no_equivalent() {
    static CMDS: [Command; 2] = [
        Command {
            note: "Windows switch drive",
            cmd: None,
            unix: None,
            windows: Some("cd /d D:\\<folder>"),
            linux: None,
            macos: None,
        },
        Command {
            note: "Standard folder enter",
            cmd: Some("cd <folder>"),
            unix: None,
            windows: None,
            linux: None,
            macos: None,
        },
    ];
    let sec = Section {
        title: "Drive Switching",
        aliases: &["switch drive", "change drive", "drive letter"],
        commands: &CMDS,
    };

    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 80,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, &sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("no_equivalent_linux", output);
}

#[test]
fn test_snapshot_placeholder_highlighting() {
    let sec = get_section("git", "Cloning a Repo");
    let ctx = RenderCtx {
        color: true,
        plain: false,
        width: 80,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("placeholder_highlighting_colored", output);
}

#[test]
fn test_snapshot_per_os_output() {
    let sec = get_section("shell", "Navigating Directories");
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 80,
    };

    let mut buf_win = Vec::new();
    render_section(&mut buf_win, sec, Os::Windows, &ctx).expect("render windows");
    let out_win = String::from_utf8(buf_win).expect("utf8");
    insta::assert_snapshot!("navigating_directories_windows", out_win);

    let mut buf_linux = Vec::new();
    render_section(&mut buf_linux, sec, Os::Linux, &ctx).expect("render linux");
    let out_linux = String::from_utf8(buf_linux).expect("utf8");
    insta::assert_snapshot!("navigating_directories_linux", out_linux);
}
