use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::info;

/// Code analyzer: understands project structure, detects languages,
/// and tracks Emir's coding patterns.
///
/// Supported domains:
/// - Roblox (Luau): .lua, .luau files
/// - Unity (C#): .cs files
/// - Unreal (C++/Blueprint): .cpp, .h, .uasset
/// - Godot (GDScript): .gd files
/// - Web: .ts, .tsx, .js, .jsx, .html, .css
/// - Mobile: .dart, .kt, .swift
/// - Rust: .rs files
/// - Python: .py files
#[allow(dead_code)]
pub struct CodeAnalyzer {
    project_root: PathBuf,
}

impl CodeAnalyzer {
    pub fn new(project_root: &str) -> Self {
        Self {
            project_root: PathBuf::from(project_root),
        }
    }

    /// Analyze a project directory and return statistics.
    pub fn analyze(&self, path: &Path) -> Result<ProjectAnalysis> {
        let mut analysis = ProjectAnalysis::default();

        self.walk_dir(path, &mut analysis)?;

        info!(
            files = analysis.total_files,
            loc = analysis.total_loc,
            languages = ?analysis.language_breakdown.keys().collect::<Vec<_>>(),
            "Analysis complete"
        );

        Ok(analysis)
    }

    fn walk_dir(&self, dir: &Path, analysis: &mut ProjectAnalysis) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip common non-source dirs
            let name = entry.file_name().to_string_lossy().to_string();
            if matches!(
                name.as_str(),
                "node_modules" | ".git" | "target" | "__pycache__" | ".venv" | "build" | "dist"
            ) {
                continue;
            }

            if path.is_dir() {
                self.walk_dir(&path, analysis)?;
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let language = detect_language(ext);
                    let loc = count_lines(&path);

                    analysis.total_files += 1;
                    analysis.total_loc += loc;
                    *analysis.language_breakdown.entry(language).or_default() += loc;
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct ProjectAnalysis {
    pub total_files: u64,
    pub total_loc: u64,
    pub language_breakdown: HashMap<String, u64>,
}

fn detect_language(ext: &str) -> String {
    match ext {
        "rs" => "Rust",
        "py" => "Python",
        "lua" | "luau" => "Luau",
        "cs" => "C#",
        "cpp" | "cc" | "cxx" | "h" | "hpp" => "C++",
        "gd" => "GDScript",
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" => "JavaScript",
        "dart" => "Dart",
        "kt" => "Kotlin",
        "swift" => "Swift",
        "html" => "HTML",
        "css" | "scss" | "sass" => "CSS",
        "yaml" | "yml" => "YAML",
        "toml" => "TOML",
        "json" => "JSON",
        "md" => "Markdown",
        _ => "Other",
    }
    .to_string()
}

fn count_lines(path: &Path) -> u64 {
    std::fs::read_to_string(path)
        .map(|c| c.lines().count() as u64)
        .unwrap_or(0)
}
