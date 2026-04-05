use regex::Regex;
use std::fmt;
use std::str::FromStr;

/// Memory type categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryType {
    User,
    Feedback,
    Project,
    Reference,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Feedback => write!(f, "feedback"),
            Self::Project => write!(f, "project"),
            Self::Reference => write!(f, "reference"),
        }
    }
}

impl FromStr for MemoryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "user" => Ok(Self::User),
            "feedback" => Ok(Self::Feedback),
            "project" => Ok(Self::Project),
            "reference" => Ok(Self::Reference),
            other => Err(format!("unknown memory type: {other}")),
        }
    }
}

/// Parsed YAML frontmatter from a memory file.
#[derive(Debug, Clone)]
pub struct MemoryFrontmatter {
    pub name: String,
    pub description: String,
    pub memory_type: MemoryType,
    pub scope: Option<String>,
}

/// A validation issue found in a memory file.
#[derive(Debug, Clone)]
pub struct Issue {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub warning: bool,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = if self.warning { "warning" } else { "error" };
        write!(f, "{}:{}: {}: {}", self.file, self.line, level, self.message)
    }
}

/// Parse YAML frontmatter from memory file content.
///
/// Expects content starting with `---`, key-value pairs, and closing `---`.
/// Returns `None` if no valid frontmatter block is found.
pub fn parse_frontmatter(content: &str) -> Option<MemoryFrontmatter> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || lines[0].trim() != "---" {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| l.trim() == "---")?;
    let end = end + 1; // adjust for the skip(1)

    let mut name = None;
    let mut description = None;
    let mut memory_type = None;
    let mut scope = None;

    for line in &lines[1..end] {
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim().to_string();
            if value.is_empty() {
                continue;
            }
            match key {
                "name" => name = Some(value),
                "description" => description = Some(value),
                "type" => memory_type = MemoryType::from_str(&value).ok(),
                "scope" => scope = Some(value),
                _ => {}
            }
        }
    }

    Some(MemoryFrontmatter {
        name: name?,
        description: description?,
        memory_type: memory_type?,
        scope,
    })
}

/// Return the line number (1-based) where the body starts (after closing `---`).
fn body_start_line(content: &str) -> Option<usize> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || lines[0].trim() != "---" {
        return None;
    }
    let end = lines.iter().skip(1).position(|l| l.trim() == "---")?;
    Some(end + 2) // 0-indexed skip(1) position -> 1-based line after closing ---
}

/// Validate a single memory file, returning any issues found.
pub fn validate_memory(path: &str, content: &str) -> Vec<Issue> {
    let mut issues = Vec::new();

    // Check frontmatter presence
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() || lines[0].trim() != "---" {
        issues.push(Issue {
            file: path.to_string(),
            line: 1,
            message: "missing frontmatter (file must start with ---)".to_string(),
            warning: false,
        });
        return issues;
    }

    let closing = lines.iter().skip(1).position(|l| l.trim() == "---");
    if closing.is_none() {
        issues.push(Issue {
            file: path.to_string(),
            line: 1,
            message: "unclosed frontmatter (missing closing ---)".to_string(),
            warning: false,
        });
        return issues;
    }

    // Parse and validate frontmatter fields
    let fm = parse_frontmatter(content);
    if fm.is_none() {
        // Determine which fields are missing
        let end = closing.unwrap() + 1;
        let mut has_name = false;
        let mut has_desc = false;
        let mut has_type = false;
        let mut bad_type: Option<String> = None;

        for line in &lines[1..end] {
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "name" if !value.is_empty() => has_name = true,
                    "description" if !value.is_empty() => has_desc = true,
                    "type" => {
                        if MemoryType::from_str(value).is_ok() {
                            has_type = true;
                        } else if !value.is_empty() {
                            bad_type = Some(value.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        if !has_name {
            issues.push(Issue {
                file: path.to_string(),
                line: 1,
                message: "frontmatter missing required field: name".to_string(),
                warning: false,
            });
        }
        if !has_desc {
            issues.push(Issue {
                file: path.to_string(),
                line: 1,
                message: "frontmatter missing required field: description".to_string(),
                warning: false,
            });
        }
        if let Some(bad) = bad_type {
            issues.push(Issue {
                file: path.to_string(),
                line: 1,
                message: format!("invalid type: {bad} (expected user/feedback/project/reference)"),
                warning: false,
            });
        } else if !has_type {
            issues.push(Issue {
                file: path.to_string(),
                line: 1,
                message: "frontmatter missing required field: type".to_string(),
                warning: false,
            });
        }

        return issues;
    }

    // Non-empty body check
    let body_line = body_start_line(content).unwrap_or(0);
    let body: String = lines
        .iter()
        .skip(body_line)
        .copied()
        .collect::<Vec<&str>>()
        .join("\n");
    if body.trim().is_empty() {
        issues.push(Issue {
            file: path.to_string(),
            line: body_line + 1,
            message: "memory body is empty (content required after frontmatter)".to_string(),
            warning: false,
        });
    }

    // Machine-local path check
    let local_path_re = Regex::new(r"(?:/home/\w+|/Users/\w+|C:\\Users\\\w+)").unwrap();
    for (i, line) in lines.iter().enumerate() {
        if local_path_re.is_match(line) {
            issues.push(Issue {
                file: path.to_string(),
                line: i + 1,
                message: "machine-local path detected".to_string(),
                warning: true,
            });
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_MEMORY: &str = "\
---
name: test-memory
description: A test memory for validation
type: feedback
scope: testing
---

Always run tests before committing.

**Why:** Prevents regressions.

**How to apply:** Use `make check` before `git commit`.
";

    #[test]
    fn parse_valid_frontmatter() {
        let fm = parse_frontmatter(VALID_MEMORY).unwrap();
        assert_eq!(fm.name, "test-memory");
        assert_eq!(fm.description, "A test memory for validation");
        assert_eq!(fm.memory_type, MemoryType::Feedback);
        assert_eq!(fm.scope.as_deref(), Some("testing"));
    }

    #[test]
    fn parse_frontmatter_no_scope() {
        let content = "\
---
name: minimal
description: Minimal memory
type: user
---

Content here.
";
        let fm = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name, "minimal");
        assert!(fm.scope.is_none());
        assert_eq!(fm.memory_type, MemoryType::User);
    }

    #[test]
    fn parse_frontmatter_missing_opening() {
        assert!(parse_frontmatter("no frontmatter here").is_none());
    }

    #[test]
    fn parse_frontmatter_missing_closing() {
        let content = "---\nname: broken\n";
        assert!(parse_frontmatter(content).is_none());
    }

    #[test]
    fn validate_valid_memory() {
        let issues = validate_memory("test.md", VALID_MEMORY);
        assert!(issues.is_empty(), "expected no issues, got: {issues:?}");
    }

    #[test]
    fn validate_missing_frontmatter() {
        let issues = validate_memory("bad.md", "no frontmatter");
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("missing frontmatter"));
        assert!(!issues[0].warning);
    }

    #[test]
    fn validate_empty_body() {
        let content = "\
---
name: empty-body
description: Has frontmatter but no body
type: project
---
";
        let issues = validate_memory("empty.md", content);
        assert!(issues.iter().any(|i| i.message.contains("body is empty")));
    }

    #[test]
    fn validate_machine_local_path() {
        let content = "\
---
name: local-path
description: Contains a local path
type: feedback
---

Use the binary at /home/alice/bin/tool.
";
        let issues = validate_memory("path.md", content);
        assert!(issues.iter().any(|i| i.message.contains("machine-local path")));
        assert!(issues.iter().any(|i| i.warning));
    }

    #[test]
    fn validate_invalid_type() {
        let content = "\
---
name: bad-type
description: Invalid memory type
type: invalid
---

Some content.
";
        let issues = validate_memory("bad-type.md", content);
        assert!(issues.iter().any(|i| i.message.contains("invalid type")));
    }

    #[test]
    fn memory_type_display() {
        assert_eq!(MemoryType::User.to_string(), "user");
        assert_eq!(MemoryType::Feedback.to_string(), "feedback");
        assert_eq!(MemoryType::Project.to_string(), "project");
        assert_eq!(MemoryType::Reference.to_string(), "reference");
    }

    #[test]
    fn memory_type_from_str() {
        assert_eq!(MemoryType::from_str("user").unwrap(), MemoryType::User);
        assert_eq!(MemoryType::from_str("FEEDBACK").unwrap(), MemoryType::Feedback);
        assert!(MemoryType::from_str("invalid").is_err());
    }
}
