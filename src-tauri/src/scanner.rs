use crate::claude_meta::{self, ClaudeMeta};
use crate::config::RecentEntry;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

const MARKERS: &[&str] = &[
    ".git",
    "CLAUDE.md",
    "package.json",
    "pyproject.toml",
    "Cargo.toml",
    ".claude",
];

const SKIP_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".venv",
    "venv",
    "__pycache__",
    "dist",
    "build",
    ".next",
    ".git",
    "worktrees",
];

// Subfolder names that are part of a larger project, not standalone
const SUBFOLDER_NAMES: &[&str] = &[
    "frontend",
    "backend",
    "client",
    "server",
    "web",
    "app",
    "api",
    "packages",
    "apps",
    "src-tauri",
    "docs",
    "services",
    "mobile",
    "desktop",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub markers: Vec<String>,
    pub pinned: bool,
    pub last_launched: Option<String>,
    pub claude: ClaudeMeta,
    pub project_type: String,
    pub location: String, // e.g. "Desktop", "Projects"
}

fn derive_location(_path: &str, scan_dir: &str) -> String {
    // Extract a short label from the scan directory
    // /Users/john/Desktop -> "Desktop"
    // /Volumes/MyDrive/Projects -> "MyDrive"
    // ~/Projects -> "Projects"
    let dir = scan_dir
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(scan_dir);

    // For /Volumes/X/Y paths, prefer the volume name
    if scan_dir.starts_with("/Volumes/") {
        if let Some(vol) = scan_dir.strip_prefix("/Volumes/") {
            if let Some(name) = vol.split('/').next() {
                return name.to_string();
            }
        }
    }

    dir.to_string()
}

fn detect_project_type(markers: &[String]) -> String {
    if markers.contains(&"Cargo.toml".to_string()) {
        "rust".to_string()
    } else if markers.contains(&"pyproject.toml".to_string()) {
        "python".to_string()
    } else if markers.contains(&"package.json".to_string()) {
        "node".to_string()
    } else {
        "other".to_string()
    }
}

pub fn scan_directories(
    scan_dirs: &[String],
    depth: usize,
    pinned: &[String],
    recents: &[RecentEntry],
) -> Vec<Project> {
    let mut seen = HashSet::new();
    let mut project_paths: Vec<String> = Vec::new();
    let mut projects = Vec::new();

    for scan_dir in scan_dirs {
        let scan_path = Path::new(scan_dir);
        if !scan_path.exists() {
            continue;
        }

        let walker = WalkDir::new(scan_dir)
            .max_depth(depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                e.depth() == 0 || !SKIP_DIRS.contains(&name.as_ref())
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_dir() {
                continue;
            }

            let dir_path = entry.path();
            let dir_str = dir_path.to_string_lossy().to_string();

            if seen.contains(&dir_str) {
                continue;
            }

            // Skip children of already-detected projects
            let is_child = project_paths
                .iter()
                .any(|pp| dir_str.starts_with(&format!("{}/", pp)));
            if is_child {
                continue;
            }

            // Skip common subfolder names if the parent has project markers
            let dir_name = dir_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if SUBFOLDER_NAMES.contains(&dir_name.as_str()) {
                if let Some(parent) = dir_path.parent() {
                    let parent_has_markers = MARKERS.iter().any(|m| parent.join(m).exists());
                    if parent_has_markers {
                        // Register the parent as a project instead
                        let parent_str = parent.to_string_lossy().to_string();
                        if !seen.contains(&parent_str) {
                            // Will be picked up when the walker visits it, or already was
                        }
                        continue;
                    }
                }
            }

            let found_markers: Vec<String> = MARKERS
                .iter()
                .filter(|m| dir_path.join(m).exists())
                .map(|m| m.to_string())
                .collect();

            if found_markers.is_empty() {
                continue;
            }

            seen.insert(dir_str.clone());
            project_paths.push(dir_str.clone());

            let name = dir_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| dir_str.clone());

            let last_launched = recents
                .iter()
                .find(|r| r.path == dir_str)
                .map(|r| r.timestamp.clone());

            let project_type = detect_project_type(&found_markers);

            let has_claude_md = found_markers.contains(&"CLAUDE.md".to_string());
            let location = derive_location(&dir_str, scan_dir);

            projects.push(Project {
                name,
                path: dir_str.clone(),
                markers: found_markers,
                pinned: pinned.contains(&dir_str),
                last_launched,
                claude: ClaudeMeta { has_claude_md, ..Default::default() },
                project_type,
                location,
            });
        }
    }

    // Load Claude metadata for all discovered paths
    let all_paths: Vec<String> = projects.iter().map(|p| p.path.clone()).collect();
    let claude_stats = claude_meta::load_project_stats(&all_paths);
    for p in &mut projects {
        if let Some(meta) = claude_stats.get(&p.path) {
            let has_claude_md = p.claude.has_claude_md;
            p.claude = meta.clone();
            p.claude.has_claude_md = has_claude_md;
        }
    }

    // Disambiguate duplicate names by prepending parent directory
    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for p in &projects {
        *name_counts.entry(p.name.to_lowercase()).or_insert(0) += 1;
    }
    for p in &mut projects {
        if name_counts.get(&p.name.to_lowercase()).copied().unwrap_or(0) > 1 {
            // Get parent directory name from path
            if let Some(parent) = Path::new(&p.path)
                .parent()
                .and_then(|pp| pp.file_name())
                .map(|n| n.to_string_lossy().to_string())
            {
                p.name = format!("{}/{}", parent, p.name);
            }
        }
    }

    // Sort: pinned first, then by Claude last_active (most recent), then alphabetical
    projects.sort_by(|a, b| {
        b.pinned
            .cmp(&a.pinned)
            .then_with(|| {
                let a_time = a
                    .claude
                    .last_active
                    .as_ref()
                    .or(a.last_launched.as_ref());
                let b_time = b
                    .claude
                    .last_active
                    .as_ref()
                    .or(b.last_launched.as_ref());
                match (b_time, a_time) {
                    (Some(bt), Some(at)) => bt.cmp(at),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            })
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    projects
}
