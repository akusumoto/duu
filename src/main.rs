use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{ArgAction, ArgGroup, Parser};

/// duu - disk usage analytics tool
#[derive(Parser)]
#[command(
    name = "duu",
    about = "Show folder sizes, largest first",
    disable_help_flag = true
)]
#[command(group(
    ArgGroup::new("unit")
        .args(["human", "kilo", "mega", "giga"])
        .multiple(false)
))]
struct Cli {
    /// Path to analyze
    path: Option<PathBuf>,

    /// Depth to display (1 = direct children only)
    #[arg(short = 'd', long = "depth", default_value_t = 1)]
    depth: usize,

    /// Human-readable sizes (default)
    #[arg(short = 'h')]
    human: bool,

    /// Sizes in KB
    #[arg(short = 'k')]
    kilo: bool,

    /// Sizes in MB
    #[arg(short = 'm')]
    mega: bool,

    /// Sizes in GB
    #[arg(short = 'g')]
    giga: bool,

    #[arg(long = "help", action = ArgAction::Help)]
    help: Option<bool>,
}

#[derive(Clone, Copy)]
enum Unit {
    Human,
    Kilo,
    Mega,
    Giga,
}

struct Entry {
    name: String,
    size: u64,
    is_dir: bool,
    children: Vec<Entry>,
}

fn scan(path: &Path) -> Entry {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    let mut total: u64 = 0;
    let mut children = Vec::new();

    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!("duu: cannot read '{}': {}", path.display(), e);
            return Entry {
                name,
                size: 0,
                is_dir: true,
                children,
            };
        }
    };

    for dir_entry in read_dir.flatten() {
        let entry_path = dir_entry.path();
        let file_type = match dir_entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            let child = scan(&entry_path);
            total += child.size;
            children.push(child);
            continue;
        }

        // Symlinks are counted by their own size, not their target's
        let meta = if file_type.is_symlink() {
            fs::symlink_metadata(&entry_path)
        } else {
            dir_entry.metadata()
        };
        if let Ok(meta) = meta {
            total += meta.len();
            children.push(Entry {
                name: dir_entry.file_name().to_string_lossy().into_owned(),
                size: meta.len(),
                is_dir: false,
                children: Vec::new(),
            });
        }
    }

    children.sort_by(|a, b| b.size.cmp(&a.size));

    Entry {
        name,
        size: total,
        is_dir: true,
        children,
    }
}

fn human_readable(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "K", "M", "G", "T", "P"];
    let mut size = bytes as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{bytes}{}", UNITS[0])
    } else {
        format!("{size:.1}{}", UNITS[idx])
    }
}

fn format_size(bytes: u64, unit: Unit) -> String {
    match unit {
        Unit::Human => human_readable(bytes),
        Unit::Kilo => format!("{}K", bytes.div_ceil(1024)),
        Unit::Mega => format!("{}M", bytes.div_ceil(1024 * 1024)),
        Unit::Giga => format!("{}G", bytes.div_ceil(1024 * 1024 * 1024)),
    }
}

/// One output line: tree-branch prefix, formatted size, and name.
struct Line {
    prefix: String,
    size: String,
    name: String,
}

fn display_name(entry: &Entry) -> String {
    if entry.is_dir && !entry.name.ends_with('/') {
        format!("{}/", entry.name)
    } else {
        entry.name.clone()
    }
}

fn collect_lines(
    entry: &Entry,
    unit: Unit,
    max_depth: usize,
    level: usize,
    indent: &str,
    lines: &mut Vec<Line>,
) {
    if level > max_depth {
        return;
    }
    let count = entry.children.len();
    for (i, child) in entry.children.iter().enumerate() {
        let is_last = i + 1 == count;
        let (branch, next) = if is_last { ("└─ ", "   ") } else { ("├─ ", "│  ") };
        lines.push(Line {
            prefix: format!("{indent}{branch}"),
            size: format_size(child.size, unit),
            name: display_name(child),
        });
        collect_lines(child, unit, max_depth, level + 1, &format!("{indent}{next}"), lines);
    }
}

fn print_tree(root: &Entry, unit: Unit, max_depth: usize) {
    let mut lines = vec![Line {
        prefix: String::new(),
        size: format_size(root.size, unit),
        name: display_name(root),
    }];
    collect_lines(root, unit, max_depth, 1, "", &mut lines);

    let width = lines.iter().map(|l| l.size.len()).max().unwrap_or(0);
    for line in &lines {
        println!("{}{:>width$} {}", line.prefix, line.size, line.name);
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let path = cli.path.unwrap_or_else(|| PathBuf::from("."));

    let unit = if cli.kilo {
        Unit::Kilo
    } else if cli.mega {
        Unit::Mega
    } else if cli.giga {
        Unit::Giga
    } else {
        Unit::Human
    };

    let metadata = match fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("duu: cannot access '{}': {}", path.display(), e);
            return ExitCode::FAILURE;
        }
    };

    let root_label = path.display().to_string();

    if metadata.is_file() {
        println!("{} {}", format_size(metadata.len(), unit), root_label);
        return ExitCode::SUCCESS;
    }

    let mut root = scan(&path);
    root.name = root_label;
    print_tree(&root, unit, cli.depth.max(1));

    ExitCode::SUCCESS
}
