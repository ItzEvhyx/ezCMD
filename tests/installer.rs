#[test]
fn windows_installer_source_stays_ascii_for_windows_powershell_5_1() {
    let script = include_bytes!("../install.ps1");
    assert!(
        script.iter().all(u8::is_ascii),
        "install.ps1 must be ASCII-only: Windows PowerShell 5.1 otherwise assumes an ANSI code page"
    );
    let script = std::str::from_utf8(script).expect("ASCII is valid UTF-8");
    assert!(script.contains("$Check = [char]0x2713"));
}
