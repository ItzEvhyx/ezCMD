//! A tiny, best-effort local frecency store.  It deliberately persists recipe
//! identifiers only; user queries and command text are never written.
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default, Clone)]
pub struct History {
    path: Option<PathBuf>,
    entries: BTreeMap<String, Entry>,
}

#[derive(Debug, Clone, Copy, Default)]
struct Entry {
    count: u32,
    last_used: u64,
}

impl History {
    pub fn load() -> Self {
        if std::env::var_os("EZCMD_NO_HISTORY").is_some() {
            return Self::default();
        }
        let Some(path) = history_path() else {
            return Self::default();
        };
        Self::from_path(path)
    }

    fn from_path(path: PathBuf) -> Self {
        let mut history = Self {
            path: Some(path.clone()),
            entries: BTreeMap::new(),
        };
        let Ok(text) = fs::read_to_string(path) else {
            return history;
        };
        for line in text.lines() {
            let mut values = line.splitn(3, '\t');
            let (Some(id), Some(count), Some(last_used)) =
                (values.next(), values.next(), values.next())
            else {
                continue;
            };
            if valid_id(id) {
                if let (Ok(count), Ok(last_used)) = (count.parse(), last_used.parse()) {
                    history
                        .entries
                        .insert(id.to_string(), Entry { count, last_used });
                }
            }
        }
        history
    }

    pub fn boost(&self, recipe_id: &str) -> f32 {
        let Some(entry) = self.entries.get(recipe_id) else {
            return 0.0;
        };
        let age_hours = now().saturating_sub(entry.last_used) / 3600;
        let recency = if age_hours < 24 {
            1.0
        } else if age_hours < 24 * 7 {
            0.5
        } else {
            0.1
        };
        let frequency = entry.count.min(10) as f32 / 10.0;
        0.5 * recency * frequency
    }

    pub fn record(&mut self, recipe_id: &str) {
        let Some(path) = &self.path else {
            return;
        };
        if !valid_id(recipe_id) {
            return;
        }
        let entry = self.entries.entry(recipe_id.to_string()).or_default();
        entry.count = entry.count.saturating_add(1);
        entry.last_used = now();
        let Some(parent) = path.parent() else {
            return;
        };
        if fs::create_dir_all(parent).is_err() {
            return;
        }
        let mut output = String::new();
        for (id, entry) in &self.entries {
            output.push_str(id);
            output.push('\t');
            output.push_str(&entry.count.to_string());
            output.push('\t');
            output.push_str(&entry.last_used.to_string());
            output.push('\n');
        }
        let _ = fs::write(path, output);
    }

    pub fn forget() -> bool {
        if std::env::var_os("EZCMD_NO_HISTORY").is_some() {
            return false;
        }
        history_path().is_some_and(|path| match fs::remove_file(path) {
            Ok(()) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
            Err(_) => false,
        })
    }
}

fn history_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("APPDATA").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("ezcmd").join("history"))
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-')
        })
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("ezcmd-history-{unique}"))
    }

    #[test]
    fn records_only_recipe_ids_and_caps_the_boost() {
        let path = temp_path();
        let mut history = History::from_path(path.clone());
        history.record("next.create");
        for _ in 0..20 {
            history.record("next.create");
        }
        let text = fs::read_to_string(&path).expect("history file");
        assert!(text.starts_with("next.create\t"));
        assert!(!text.contains("how do i"));
        assert!((0.0..=0.5).contains(&history.boost("next.create")));
        fs::remove_file(path).expect("cleanup");
    }

    #[test]
    fn malformed_and_unsafe_ids_are_ignored() {
        let path = temp_path();
        let mut history = History::from_path(path.clone());
        history.record("bad\tquery");
        assert!(!path.exists());
        assert!(!valid_id("bad\nquery"));
    }
}
