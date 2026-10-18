use ezcmd::content::{all_tools, CATEGORIES};
use ezcmd::model::{Command, Section, Tool};
use ezcmd::os::Os;
use ezcmd::render::{render_section, toc_overview, toc_tool, usage_guide, RenderCtx};

fn get_tool(tool_name: &str) -> &'static Tool {
    all_tools()
        .find(|tool| tool.slug == tool_name)
        .unwrap_or_else(|| panic!("tool {tool_name} not found"))
}

fn get_section<'a>(tool: &'a Tool, title: &str) -> &'a Section {
    tool.sections()
        .find(|s| s.title == title)
        .unwrap_or_else(|| panic!("section {title} not found"))
}

#[test]
fn test_snapshot_wide_layout() {
    let tool = get_tool("git");
    let sec = get_section(tool, "Initializing a Repo");
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 100,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, tool, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("wide_layout_no_color", output);
}

#[test]
fn test_snapshot_wide_layout_colored() {
    let tool = get_tool("git");
    let sec = get_section(tool, "Initializing a Repo");
    let ctx = RenderCtx {
        color: true,
        plain: false,
        width: 100,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, tool, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("wide_layout_colored", output);
}

#[test]
fn test_snapshot_narrow_layout() {
    let tool = get_tool("git");
    let sec = get_section(tool, "Initializing a Repo");
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 60,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, tool, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("narrow_layout", output);
}

#[test]
fn test_snapshot_plain_mode() {
    let tool = get_tool("git");
    let sec = get_section(tool, "Initializing a Repo");
    let ctx = RenderCtx {
        color: false,
        plain: true,
        width: 80,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, tool, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("plain_mode", output);
}

#[test]
fn test_snapshot_omits_unavailable_commands() {
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
    let tool = Tool {
        slug: "test",
        name: "Test Tool",
        category: "Test",
        aliases: &[],
        flag_aliases: &[],
        platforms: &[],
        notes: &[],
        groups: &[],
    };

    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 80,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, &tool, &sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("omits_unavailable_commands_linux", output);
}

#[test]
fn test_snapshot_placeholder_highlighting() {
    let tool = get_tool("git");
    let sec = get_section(tool, "Cloning a Repo");
    let ctx = RenderCtx {
        color: true,
        plain: false,
        width: 80,
    };
    let mut buf = Vec::new();
    render_section(&mut buf, tool, sec, Os::Linux, &ctx).expect("render");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("placeholder_highlighting_colored", output);
}

#[test]
fn test_snapshot_per_os_output() {
    let tool = get_tool("shell");
    let sec = get_section(tool, "Navigating Directories");
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 80,
    };

    let mut buf_win = Vec::new();
    render_section(&mut buf_win, tool, sec, Os::Windows, &ctx).expect("render windows");
    let out_win = String::from_utf8(buf_win).expect("utf8");
    insta::assert_snapshot!("navigating_directories_windows", out_win);

    let mut buf_linux = Vec::new();
    render_section(&mut buf_linux, tool, sec, Os::Linux, &ctx).expect("render linux");
    let out_linux = String::from_utf8(buf_linux).expect("utf8");
    insta::assert_snapshot!("navigating_directories_linux", out_linux);
}

#[test]
fn test_snapshot_toc_overview_plain() {
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 100,
    };
    let mut buf = Vec::new();
    toc_overview(&mut buf, CATEGORIES, Os::Linux, &ctx).expect("render overview");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("toc_overview_plain", output);
}

#[test]
fn test_snapshot_toc_tool_single_group_plain() {
    let tool = all_tools()
        .find(|tool| tool.slug == "sqlite")
        .expect("sqlite tool");
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 100,
    };
    let mut buf = Vec::new();
    toc_tool(&mut buf, tool, Os::Linux, &ctx).expect("render tool toc");
    let output = String::from_utf8(buf).expect("utf8");
    insta::assert_snapshot!("toc_tool_single_group_plain", output);
    assert!(!output.contains("sqlite3 "));
    assert!(!output.contains("Everything SQLite\n"));
}

#[test]
fn test_usage_guide_narrow_drops_examples() {
    let ctx = RenderCtx {
        color: false,
        plain: false,
        width: 60,
    };
    let mut buf = Vec::new();
    usage_guide(&mut buf, &ctx).expect("render guide");
    let output = String::from_utf8(buf).expect("utf8");
    assert!(output.contains("HOW TO USE"));
    assert!(!output.contains("ezcmd --help -docker"));
}
