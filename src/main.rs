use agent_memories::{parse_frontmatter, validate_memory, write_memory, MemoryType};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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
    /// Write a new memory file
    Write {
        /// Memory name (used in frontmatter and to generate filename)
        #[arg(long)]
        name: String,
        /// Short description of the memory
        #[arg(long)]
        description: String,
        /// Memory type (user, feedback, project, reference)
        #[arg(long, rename_all = "lower", value_name = "TYPE")]
        r#type: String,
        /// Optional scope qualifier
        #[arg(long)]
        scope: Option<String>,
        /// Memory body text (mutually exclusive with --file)
        #[arg(long, conflicts_with = "file")]
        body: Option<String>,
        /// Path to a file containing the memory body (mutually exclusive with --body)
        #[arg(long, conflicts_with = "body")]
        file: Option<PathBuf>,
        /// Directory to write the memory file into (defaults to current dir)
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Validate { file } => cmd_validate(&file),
        Command::List { dir } => cmd_list(&dir),
        Command::Write {
            name,
            description,
            r#type,
            scope,
            body,
            file,
            dir,
        } => cmd_write(&name, &description, &r#type, scope.as_deref(), body, file, &dir),
    }
}

fn cmd_validate(file: &Path) -> Result<()> {
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

fn cmd_write(
    name: &str,
    description: &str,
    type_str: &str,
    scope: Option<&str>,
    body: Option<String>,
    file: Option<PathBuf>,
    dir: &Path,
) -> Result<()> {
    let memory_type = MemoryType::from_str(type_str)
        .map_err(|e| anyhow::anyhow!(e))?;

    let body_text = match (body, file) {
        (Some(b), _) => b,
        (_, Some(f)) => fs::read_to_string(&f)
            .with_context(|| format!("failed to read body file {}", f.display()))?,
        (None, None) => anyhow::bail!("either --body or --file must be provided"),
    };

    let path = write_memory(dir, name, description, memory_type, scope, &body_text)?;
    println!("{}", path.display());
    Ok(())
}

fn cmd_list(dir: &Path) -> Result<()> {
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
