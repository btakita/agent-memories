use agent_memories::{parse_frontmatter, validate_memory};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "agent-memories", about = "Validate and inspect agent memory files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a single memory file
    Validate {
        /// Path to the memory file
        file: PathBuf,
    },
    /// List all memories in a directory with type and name
    List {
        /// Directory containing memory files
        dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { file } => cmd_validate(&file),
        Command::List { dir } => cmd_list(&dir),
    }
}

fn cmd_validate(file: &PathBuf) -> Result<()> {
    let path_str = file.display().to_string();
    let content = fs::read_to_string(file)
        .with_context(|| format!("failed to read {path_str}"))?;
    let issues = validate_memory(&path_str, &content);

    if issues.is_empty() {
        println!("{path_str}: ok");
    } else {
        for issue in &issues {
            println!("{issue}");
        }
        std::process::exit(1);
    }
    Ok(())
}

fn cmd_list(dir: &PathBuf) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "md")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let filename = path.file_name().unwrap_or_default().to_string_lossy();
        match parse_frontmatter(&content) {
            Some(fm) => {
                let scope = fm.scope.as_deref().unwrap_or("-");
                println!("{:<12} {:<20} {}", fm.memory_type, fm.name, scope);
            }
            None => {
                println!("{:<12} {:<20} (no frontmatter)", "-", filename);
            }
        }
    }
    Ok(())
}
