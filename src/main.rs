use clap::{Arg, Command};
use std::process::Command as ProcessCommand;
use std::str;

#[derive(Debug)]
struct CommitInfo {
    hash: String,
    date: String,
    author: String,
    message: String,
}

#[derive(Debug)]
struct LineChange {
    line_number: usize,
    change_type: ChangeType,
    content: String,
}

#[derive(Debug)]
enum ChangeType {
    Added,
    Removed,
    Modified,
}

fn main() {
    let matches = Command::new("garch")
        .about("Explore the evolution of code through git history")
        .subcommand(
            Command::new("lines")
                .about("Trace the evolution of specific lines in a file")
                .arg(
                    Arg::new("file_range")
                        .help("File and line range (e.g., src/main.rs:10-20)")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            Command::new("file")
                .about("Show the evolution of an entire file")
                .arg(
                    Arg::new("file_path")
                        .help("Path to the file")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("lines", sub_matches)) => {
            let file_range = sub_matches.get_one::<String>("file_range").unwrap();
            handle_lines_command(file_range);
        }
        Some(("file", sub_matches)) => {
            let file_path = sub_matches.get_one::<String>("file_path").unwrap();
            handle_file_command(file_path);
        }
        _ => {
            println!("Use 'garch --help' for usage information");
        }
    }
}

fn handle_lines_command(file_range: &str) {
    let (file_path, start_line, end_line) = parse_file_range(file_range);
    
    println!("📜 Evolution of {}:{}-{}\n", file_path, start_line, end_line);
    
    match get_line_history(&file_path, start_line, end_line) {
        Ok(commits) => {
            display_commit_history(&commits, &file_path, start_line, end_line);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn handle_file_command(file_path: &str) {
    println!("📜 Evolution of {}\n", file_path);
    
    match get_file_history(file_path) {
        Ok(commits) => {
            display_file_history(&commits, file_path);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn parse_file_range(file_range: &str) -> (String, usize, usize) {
    if let Some(colon_pos) = file_range.rfind(':') {
        let file_path = file_range[..colon_pos].to_string();
        let range_part = &file_range[colon_pos + 1..];
        
        if let Some(dash_pos) = range_part.find('-') {
            let start_line: usize = range_part[..dash_pos].parse().unwrap_or(1);
            let end_line: usize = range_part[dash_pos + 1..].parse().unwrap_or(start_line);
            (file_path, start_line, end_line)
        } else {
            let line_num: usize = range_part.parse().unwrap_or(1);
            (file_path, line_num, line_num)
        }
    } else {
        (file_range.to_string(), 1, usize::MAX)
    }
}

fn get_line_history(file_path: &str, start_line: usize, end_line: usize) -> Result<Vec<CommitInfo>, String> {
    let range = format!("{},{}", start_line, end_line);
    let output = ProcessCommand::new("git")
        .args([
            "log",
            "-L", &format!("{}:{}", range, file_path),
            "--pretty=format:%H|%ad|%an|%s",
            "--date=short",
        ])
        .output()
        .map_err(|e| format!("Failed to run git command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Git command failed: {}", 
            str::from_utf8(&output.stderr).unwrap_or("unknown error")));
    }

    let output_str = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in git output: {}", e))?;

    let mut commits = Vec::new();
    for line in output_str.lines() {
        if line.contains('|') {
            if let Some(commit) = parse_commit_line(line) {
                commits.push(commit);
            }
        }
    }

    Ok(commits)
}

fn get_file_history(file_path: &str) -> Result<Vec<CommitInfo>, String> {
    let output = ProcessCommand::new("git")
        .args([
            "log",
            "--follow",
            "--pretty=format:%H|%ad|%an|%s",
            "--date=short",
            "--",
            file_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run git command: {}", e))?;

    if !output.status.success() {
        return Err(format!("Git command failed: {}", 
            str::from_utf8(&output.stderr).unwrap_or("unknown error")));
    }

    let output_str = str::from_utf8(&output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in git output: {}", e))?;

    let commits: Vec<CommitInfo> = output_str
        .lines()
        .filter_map(parse_commit_line)
        .collect();

    Ok(commits)
}

fn parse_commit_line(line: &str) -> Option<CommitInfo> {
    let parts: Vec<&str> = line.split('|').collect();
    if parts.len() >= 4 {
        Some(CommitInfo {
            hash: parts[0].to_string(),
            date: parts[1].to_string(),
            author: abbreviate_author(parts[2]),
            message: parts[3..].join("|"), // In case message contains |
        })
    } else {
        None
    }
}

fn abbreviate_author(author: &str) -> String {
    let parts: Vec<&str> = author.split_whitespace().collect();
    if parts.len() >= 2 {
        format!("{} {}.", parts[0], parts[1].chars().next().unwrap_or('?'))
    } else {
        author.to_string()
    }
}

fn display_commit_history(commits: &[CommitInfo], file_path: &str, start_line: usize, end_line: usize) {
    for (i, commit) in commits.iter().enumerate() {
        let connector = if i == commits.len() - 1 { "└─" } else { "├─" };
        
        println!("{} Commit {} ({}) - {}", 
            connector,
            &commit.hash[..7],
            commit.date,
            commit.author
        );
        println!("│  \"{}\"", commit.message);
        
        // Get the actual changes for this commit
        if let Ok(changes) = get_commit_changes(&commit.hash, file_path, start_line, end_line) {
            for change in changes {
                display_change(&change);
            }
        }
        
        if i < commits.len() - 1 {
            println!("│");
        }
    }
}

fn display_file_history(commits: &[CommitInfo], _file_path: &str) {
    println!("Recent commits affecting this file:\n");
    
    for commit in commits.iter().take(10) { // Show last 10 commits
        println!("{} │ {} │ {}", 
            commit.date,
            commit.author,
            commit.message
        );
    }
    
    if commits.len() > 10 {
        println!("... and {} more commits", commits.len() - 10);
    }
}

fn get_commit_changes(commit_hash: &str, file_path: &str, start_line: usize, end_line: usize) -> Result<Vec<LineChange>, String> {
    let range = format!("{},{}", start_line, end_line);
    let output = ProcessCommand::new("git")
        .args([
            "show",
            commit_hash,
            "-L", &format!("{}:{}", range, file_path),
            "--no-patch",
        ])
        .output()
        .map_err(|e| format!("Failed to run git show: {}", e))?;

    // For now, return empty changes - we'd need to parse the diff output
    // This is where we'd implement the actual diff parsing
    Ok(vec![])
}

fn display_change(change: &LineChange) {
    let prefix = match change.change_type {
        ChangeType::Added => "│  +",
        ChangeType::Removed => "│  -",
        ChangeType::Modified => "│  ~",
    };
    
    println!("{} {}", prefix, change.content);
}