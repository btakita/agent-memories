use crate::MemoryType;
use anyhow::{bail, Result};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Generate a filename from a memory name and type.
///
/// Replaces spaces and non-alphanumeric characters with underscores,
/// prepends the type prefix, and appends `.md`.
pub fn generate_filename(name: &str, memory_type: &MemoryType) -> String {
    let slug: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '_' })
        .collect();
    // Collapse consecutive underscores and trim leading/trailing
    let mut collapsed = String::new();
    let mut prev_underscore = true; // treat start as underscore to trim leading
    for c in slug.chars() {
        if c == '_' {
            if !prev_underscore {
                collapsed.push('_');
            }
            prev_underscore = true;
        } else {
            collapsed.push(c);
            prev_underscore = false;
        }
    }
    let collapsed = collapsed.trim_end_matches('_');
    format!("{memory_type}_{collapsed}.md")
}

/// Format a memory file with YAML frontmatter and body.
fn format_memory(
    name: &str,
    description: &str,
    memory_type: &MemoryType,
    scope: Option<&str>,
    body: &str,
) -> String {
    let mut out = String::new();
    writeln!(out, "---").unwrap();
    writeln!(out, "name: {name}").unwrap();
    writeln!(out, "description: {description}").unwrap();
    writeln!(out, "type: {memory_type}").unwrap();
    if let Some(scope) = scope {
        writeln!(out, "scope: {scope}").unwrap();
    }
    writeln!(out, "---").unwrap();
    writeln!(out).unwrap();
    write!(out, "{body}").unwrap();
    if !body.ends_with('\n') {
        writeln!(out).unwrap();
    }
    out
}

/// Write a new memory file to the memories directory.
///
/// Creates the file at `<memories_dir>/<filename>.md` with proper frontmatter.
/// Returns the path of the written file.
pub fn write_memory(
    memories_dir: &Path,
    name: &str,
    description: &str,
    memory_type: MemoryType,
    scope: Option<&str>,
    body: &str,
) -> Result<PathBuf> {
    let filename = generate_filename(name, &memory_type);
    let path = memories_dir.join(&filename);

    if path.exists() {
        bail!("memory file already exists: {}", path.display());
    }

    std::fs::create_dir_all(memories_dir)?;
    let content = format_memory(name, description, &memory_type, scope, body);
    std::fs::write(&path, &content)?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_frontmatter;

    #[test]
    fn generates_correct_filename() {
        assert_eq!(
            generate_filename("use agent-doc init", &MemoryType::Feedback),
            "feedback_use_agent_doc_init.md"
        );
        assert_eq!(
            generate_filename("My Cool Project", &MemoryType::Project),
            "project_my_cool_project.md"
        );
        assert_eq!(
            generate_filename("  spaces  everywhere  ", &MemoryType::User),
            "user_spaces_everywhere.md"
        );
    }

    #[test]
    fn write_memory_creates_file_with_correct_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_memory(
            dir.path(),
            "test lesson",
            "Always verify before commit",
            MemoryType::Feedback,
            None,
            "Run `make check` before committing.\n",
        )
        .unwrap();

        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        let fm = parse_frontmatter(&content).unwrap();
        assert_eq!(fm.name, "test lesson");
        assert_eq!(fm.description, "Always verify before commit");
        assert_eq!(fm.memory_type, MemoryType::Feedback);
        assert!(fm.scope.is_none());
        assert!(content.contains("Run `make check` before committing."));
    }

    #[test]
    fn write_memory_rejects_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        write_memory(
            dir.path(),
            "dup test",
            "First write",
            MemoryType::Feedback,
            None,
            "Body.\n",
        )
        .unwrap();

        let result = write_memory(
            dir.path(),
            "dup test",
            "Second write",
            MemoryType::Feedback,
            None,
            "Body 2.\n",
        );
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("already exists"),
            "error should mention file already exists"
        );
    }

    #[test]
    fn write_memory_with_scope() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_memory(
            dir.path(),
            "scoped memory",
            "Has a scope field",
            MemoryType::Project,
            Some("agent-doc"),
            "Scoped content.\n",
        )
        .unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        let fm = parse_frontmatter(&content).unwrap();
        assert_eq!(fm.scope.as_deref(), Some("agent-doc"));
    }
}
