use crate::content::context_policy;
use crate::model::PackageManager;
use std::fs;
use std::path::{Path, PathBuf};

/// Filesystem-only context.  All probes are best-effort and bounded; this
/// module deliberately has no process-spawning APIs.
#[derive(Debug, Clone)]
pub struct Context {
    pub git_repo: bool,
    pub has_remote_origin: bool,
    pub package_manager: &'static str,
    pub ecosystems: Vec<&'static str>,
}

impl Context {
    pub fn detect(cwd: &Path) -> Self {
        let mut git_repo = false;
        let mut has_remote_origin = false;
        let mut package_manager = None;
        let mut ecosystems = Vec::new();
        for path in parents(cwd).take(9) {
            let git = path.join(".git");
            if !git_repo && git.exists() {
                git_repo = true;
                let config = git_config_path(&git);
                if let Ok(bytes) = fs::read(&config) {
                    let capped = &bytes[..bytes.len().min(64 * 1024)];
                    has_remote_origin =
                        String::from_utf8_lossy(capped).contains("[remote \"origin\"]");
                }
            }
            if package_manager.is_none() {
                package_manager = context_policy()
                    .package_managers
                    .iter()
                    .find_map(|manager| {
                        manager
                            .markers
                            .iter()
                            .any(|marker| path.join(marker).exists())
                            .then_some(manager.id)
                    });
            }
            for (ecosystem, markers) in [
                ("node", &["package.json"][..]),
                ("rust", &["Cargo.toml"][..]),
                ("flutter", &["pubspec.yaml"][..]),
                (
                    "docker",
                    &["Dockerfile", "compose.yaml", "docker-compose.yml"][..],
                ),
            ] {
                if !ecosystems.contains(&ecosystem)
                    && markers.iter().any(|marker| path.join(marker).exists())
                {
                    ecosystems.push(ecosystem);
                }
            }
        }
        Self {
            git_repo,
            has_remote_origin,
            package_manager: package_manager.unwrap_or(context_policy().default_package_manager),
            ecosystems,
        }
    }

    fn package(&self) -> &'static PackageManager {
        context_policy()
            .package_managers
            .iter()
            .find(|manager| manager.id == self.package_manager)
            .expect("context policy default package manager must exist")
    }

    pub fn package_exec(&self) -> &'static str {
        self.package().exec
    }

    pub fn package_run(&self) -> &'static str {
        self.package().run
    }

    pub fn package_install(&self) -> &'static str {
        self.package().install
    }

    pub fn skip_note(&self, command: &str) -> Option<&'static str> {
        context_policy().skip_rules.iter().find_map(|rule| {
            let command_matches =
                command == rule.command || command.starts_with(&format!("{} ", rule.command));
            (command_matches && self.signal(rule.signal)).then_some(rule.note)
        })
    }

    /// Context can break ties but never create a match.  Package-manager and
    /// ecosystem names are mapped to the existing tool slugs here so recipes
    /// remain data-defined.
    pub fn matches_tool(&self, tool: &str) -> bool {
        tool == self.package_manager
            || self.ecosystems.iter().any(|ecosystem| match *ecosystem {
                "node" => matches!(tool, "npm" | "pnpm" | "yarn" | "bun" | "nextjs" | "vite"),
                "rust" => tool == "cargo",
                "flutter" => tool == "flutter",
                "docker" => tool == "docker",
                _ => false,
            })
    }

    fn signal(&self, signal: &str) -> bool {
        match signal {
            "git_repo" => self.git_repo,
            "has_remote_origin" => self.has_remote_origin,
            _ => false,
        }
    }
}

fn git_config_path(git: &Path) -> PathBuf {
    if git.is_dir() {
        return git.join("config");
    }
    // A worktree's .git is a text file containing `gitdir: <path>`.  Read only
    // a tiny bounded prefix and treat malformed data as an absent config.
    let Ok(bytes) = fs::read(git) else {
        return git.join("config");
    };
    let text = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]);
    let Some(location) = text.trim().strip_prefix("gitdir:") else {
        return git.join("config");
    };
    let path = PathBuf::from(location.trim());
    let path = if path.is_absolute() {
        path
    } else {
        git.parent().unwrap_or(Path::new(".")).join(path)
    };
    path.join("config")
}

fn parents(cwd: &Path) -> impl Iterator<Item = PathBuf> + '_ {
    std::iter::successors(Some(cwd.to_path_buf()), |path| {
        path.parent().map(Path::to_path_buf)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ezcmd-{label}-{unique}"));
        fs::create_dir_all(&path).expect("temp directory");
        path
    }

    #[test]
    fn detects_nearest_package_manager_and_git_origin() {
        let root = temp_dir("context");
        fs::write(root.join("pnpm-lock.yaml"), "lockfileVersion: '9.0'").expect("lockfile");
        fs::create_dir(root.join(".git")).expect("git directory");
        fs::write(
            root.join(".git/config"),
            "[remote \"origin\"]\nurl = example",
        )
        .expect("config");
        let child = root.join("nested");
        fs::create_dir(&child).expect("child");
        let context = Context::detect(&child);
        assert!(context.git_repo);
        assert!(context.has_remote_origin);
        assert_eq!(context.package_manager, "pnpm");
        assert!(context.ecosystems.is_empty());
        assert_eq!(context.package_exec(), "pnpm dlx");
        assert_eq!(context.package_run(), "pnpm");
        assert_eq!(context.package_install(), "pnpm add");
        assert_eq!(
            context.skip_note("git init"),
            Some("skipped: git init (already a git repo)")
        );
        assert_eq!(
            context.skip_note("git remote add origin https://example.com/x"),
            Some("skipped: git remote add origin (origin already exists)")
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_project_ecosystem_markers() {
        let root = temp_dir("ecosystems");
        fs::write(root.join("package.json"), "{}").expect("package marker");
        fs::write(root.join("Cargo.toml"), "[package]").expect("rust marker");
        fs::write(root.join("pubspec.yaml"), "name: sample").expect("flutter marker");
        fs::write(root.join("Dockerfile"), "FROM scratch").expect("docker marker");
        let context = Context::detect(&root);
        assert!(context.matches_tool("nextjs"));
        assert!(context.matches_tool("cargo"));
        assert!(context.matches_tool("flutter"));
        assert!(context.matches_tool("docker"));
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn detects_origin_from_a_git_worktree_file() {
        let root = temp_dir("worktree");
        let metadata = root.join("metadata");
        fs::create_dir(&metadata).expect("metadata");
        fs::write(root.join(".git"), "gitdir: metadata\n").expect("gitfile");
        fs::write(metadata.join("config"), "[remote \"origin\"]\n").expect("config");
        let context = Context::detect(&root);
        assert!(context.git_repo);
        assert!(context.has_remote_origin);
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn context_module_never_spawns_a_process() {
        let forbidden = ["std::process", "::Command"].concat();
        assert!(!include_str!("context.rs").contains(&forbidden));
    }
}
