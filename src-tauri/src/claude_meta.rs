use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeMeta {
    pub session_count: usize,
    pub last_active: Option<String>,
    pub last_active_ago: Option<String>,
    pub message_count: usize,
    pub has_claude_md: bool,
    pub has_memory: bool,
    pub has_tasks: bool,
}

#[derive(Deserialize)]
struct HistoryEntry {
    timestamp: Option<u64>,
    project: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

/// Encode a filesystem path to Claude's project directory name format.
/// /Users/john/my-project -> -Users-john-my-project
fn encode_path_for_claude(path: &str) -> String {
    path.replace('/', "-")
}

/// Load Claude metadata for a set of known project paths.
/// Uses history.jsonl for activity stats and ~/.claude/projects/ for session counts.
pub fn load_project_stats(known_paths: &[String]) -> HashMap<String, ClaudeMeta> {
    let mut stats: HashMap<String, ProjectAccumulator> = HashMap::new();

    // Initialize accumulators for all known paths
    for path in known_paths {
        stats.entry(path.clone()).or_default();
    }

    // 1. Parse history.jsonl — has real paths, no decoding needed
    let history_path = claude_dir().join("history.jsonl");
    if let Ok(file) = fs::File::open(&history_path) {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let Ok(line) = line else { continue };
            let Ok(entry) = serde_json::from_str::<HistoryEntry>(&line) else { continue };
            let Some(project) = entry.project else { continue };

            let acc = stats.entry(project).or_default();
            acc.message_count += 1;

            if let Some(sid) = entry.session_id {
                acc.sessions.insert(sid);
            }
            if let Some(ts) = entry.timestamp {
                if ts > acc.last_timestamp {
                    acc.last_timestamp = ts;
                }
            }
        }
    }

    // 2. Scan ~/.claude/projects/ for session files, matching by encoded path
    let projects_dir = claude_dir().join("projects");
    if projects_dir.exists() {
        for path in known_paths {
            let encoded = encode_path_for_claude(path);
            let project_dir = projects_dir.join(&encoded);

            if !project_dir.exists() {
                continue;
            }

            let acc = stats.entry(path.clone()).or_default();

            // Count .jsonl session files (UUIDs are 36+ chars)
            if let Ok(files) = fs::read_dir(&project_dir) {
                for f in files.filter_map(|f| f.ok()) {
                    let name = f.file_name().to_string_lossy().to_string();
                    if name.ends_with(".jsonl") && name.len() > 30 {
                        acc.sessions.insert(name.trim_end_matches(".jsonl").to_string());
                    }
                }
            }

            // Check for memory directory
            if project_dir.join("memory").exists() {
                acc.has_memory = true;
            }
        }
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    stats
        .into_iter()
        .map(|(path, acc)| {
            let last_active = if acc.last_timestamp > 0 {
                let secs = (acc.last_timestamp / 1000) as i64;
                chrono::DateTime::from_timestamp(secs, 0).map(|d| d.to_rfc3339())
            } else {
                None
            };

            let last_active_ago = if acc.last_timestamp > 0 {
                Some(time_ago(now, acc.last_timestamp))
            } else {
                None
            };

            let meta = ClaudeMeta {
                session_count: acc.sessions.len(),
                last_active,
                last_active_ago,
                message_count: acc.message_count,
                has_claude_md: false, // Set by scanner
                has_memory: acc.has_memory,
                has_tasks: false,
            };
            (path, meta)
        })
        .collect()
}

fn claude_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".claude")
}

fn time_ago(now_ms: u64, then_ms: u64) -> String {
    let diff_secs = (now_ms.saturating_sub(then_ms)) / 1000;
    let diff_mins = diff_secs / 60;
    let diff_hours = diff_mins / 60;
    let diff_days = diff_hours / 24;

    if diff_mins < 1 {
        "just now".to_string()
    } else if diff_mins < 60 {
        format!("{}m ago", diff_mins)
    } else if diff_hours < 24 {
        format!("{}h ago", diff_hours)
    } else if diff_days < 30 {
        format!("{}d ago", diff_days)
    } else {
        format!("{}mo ago", diff_days / 30)
    }
}

#[derive(Default)]
struct ProjectAccumulator {
    sessions: std::collections::HashSet<String>,
    message_count: usize,
    last_timestamp: u64,
    has_memory: bool,
}
