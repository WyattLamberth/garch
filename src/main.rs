use clap::{Arg, Command};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, MouseEventKind},
    execute,
    style::{Color, ResetColor, SetForegroundColor, SetBackgroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::process::Command as ProcessCommand;
use std::str;
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Style};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

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

#[derive(Debug, Clone)]
struct BlameLine {
    line_number: usize,
    author: String,
    date: String,
    commit_hash: String,
    commit_message: String,
    content: String,
    highlighted_content: String,
}

#[derive(Debug)]
struct FileVersion {
    commit_hash: String,
    commit_date: String,
    commit_message: String,
    blame_lines: Vec<BlameLine>,
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
                        .index(1)
                )
                .arg(
                    Arg::new("reverse")
                        .help("Show newest first instead of oldest first")
                        .long("reverse")
                        .short('r')
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("file")
                .about("Show the evolution of an entire file")
                .arg(
                    Arg::new("file_path")
                        .help("Path to the file")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("reverse")
                        .help("Show newest first instead of oldest first")
                        .long("reverse")
                        .short('r')
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .get_matches();

    match matches.subcommand() {
        Some(("lines", sub_matches)) => {
            let file_range = sub_matches.get_one::<String>("file_range").unwrap();
            let reverse = sub_matches.get_flag("reverse");
            handle_lines_command(file_range, reverse);
        }
        Some(("file", sub_matches)) => {
            let file_path = sub_matches.get_one::<String>("file_path").unwrap();
            let reverse = sub_matches.get_flag("reverse");
            handle_file_command(file_path, reverse);
        }
        _ => {
            println!("Use 'garch --help' for usage information");
        }
    }
}

fn handle_lines_command(file_range: &str, reverse: bool) {
    let (file_path, start_line, end_line) = parse_file_range(file_range);
    
    match get_line_history(&file_path, start_line, end_line) {
        Ok(commits) => {
            if commits.is_empty() {
                println!("No history found for {}:{}-{}", file_path, start_line, end_line);
                return;
            }
            
            // Run interactive viewer for line range by building file versions
            match get_file_versions(&file_path) {
                Ok(mut versions) => {
                    // By default, show oldest first (reverse the git log order)
                    // If reverse flag is set, keep newest first
                    if !reverse {
                        versions.reverse();
                    }
                    
                    if let Err(e) = run_interactive_viewer(&file_path, versions, start_line, end_line) {
                        eprintln!("Error running interactive viewer: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error getting file versions: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error getting line history: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_file_command(file_path: &str, reverse: bool) {
    println!("Loading file history for {}...", file_path);
    
    match get_file_versions(file_path) {
        Ok(mut versions) => {
            if versions.is_empty() {
                println!("No git history found for {}", file_path);
                return;
            }
            
            // Apply reverse ordering if requested
            if !reverse {
                versions.reverse(); // By default, show oldest first
            }
            
            match run_interactive_viewer(file_path, versions, 1, usize::MAX) {
                Ok(_) => {},
                Err(e) => eprintln!("Error running interactive viewer: {}", e),
            }
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

fn format_timestamp(timestamp: i64) -> String {
    // Simple timestamp formatting - in a real app you'd use chrono
    use std::time::{UNIX_EPOCH, Duration};
    
    if let Some(datetime) = UNIX_EPOCH.checked_add(Duration::from_secs(timestamp as u64)) {
        let days = datetime.duration_since(UNIX_EPOCH).unwrap().as_secs() / 86400;
        
        // Very rough date calculation - just for demo
        let year = 1970 + (days / 365);
        let day_of_year = days % 365;
        let month = (day_of_year / 30) + 1;
        let day = (day_of_year % 30) + 1;
        
        return format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(31));
    }
    
    "unknown".to_string()
}

fn find_closest_line_in_filtered(filtered_lines: &[&BlameLine], target_line: usize) -> Option<usize> {
    if filtered_lines.is_empty() {
        return None;
    }
    
    // Try to find the exact line first
    if let Some(pos) = filtered_lines.iter().position(|line| line.line_number == target_line) {
        return Some(pos);
    }
    
    // Find the closest line by minimum distance
    let mut closest_pos = 0;
    let mut min_distance = usize::MAX;
    
    for (pos, line) in filtered_lines.iter().enumerate() {
        let distance = if line.line_number > target_line {
            line.line_number - target_line
        } else {
            target_line - line.line_number
        };
        
        if distance < min_distance {
            min_distance = distance;
            closest_pos = pos;
        }
    }
    
    Some(closest_pos)
}

fn get_current_target_line(filtered_lines: &[&BlameLine], scroll_offset: usize, content_height: usize) -> usize {
    // Get the line number of the first visible line, or middle if multiple lines visible
    let visible_start = scroll_offset;
    let visible_end = (scroll_offset + content_height / 2).min(filtered_lines.len());
    
    if let Some(line) = filtered_lines.get(visible_start.max(visible_end.saturating_sub(1))) {
        line.line_number
    } else if let Some(line) = filtered_lines.first() {
        line.line_number
    } else {
        1 // fallback
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
            std::str::from_utf8(&output.stderr).unwrap_or("unknown error")));
    }

    let output_str = std::str::from_utf8(&output.stdout)
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
            std::str::from_utf8(&output.stderr).unwrap_or("unknown error")));
    }

    let output_str = std::str::from_utf8(&output.stdout)
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
            author: parts[2].to_string(),
            message: parts[3].to_string(),
        })
    } else {
        None
    }
}

fn get_author_color(author: &str) -> Color {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    author.hash(&mut hasher);
    let hash = hasher.finish();
    
    let colors = [
        Color::Red,
        Color::DarkCyan,
        Color::DarkGreen,
        Color::DarkYellow,
        Color::DarkBlue,
        Color::DarkMagenta,
        Color::DarkRed,
    ];
    colors[hash as usize % colors.len()]
}

fn abbreviate_author(author: &str) -> String {
    let parts: Vec<&str> = author.split_whitespace().collect();
    if parts.len() >= 2 {
        format!("{} {}.", parts[0], parts[1].chars().next().unwrap_or('?'))
    } else {
        author.to_string()
    }
}

fn get_file_versions(file_path: &str) -> Result<Vec<FileVersion>, String> {
    let commits = get_file_history(file_path)?;
    let mut versions = Vec::new();
    
    for commit in commits {
        match get_blame_for_commit(&commit.hash, file_path) {
            Ok(blame_lines) => {
                versions.push(FileVersion {
                    commit_hash: commit.hash.clone(),
                    commit_date: commit.date,
                    commit_message: commit.message,
                    blame_lines,
                });
            }
            Err(_) => continue, // Skip commits where we can't get blame
        }
    }
    
    Ok(versions)
}

fn get_blame_for_commit(commit_hash: &str, file_path: &str) -> Result<Vec<BlameLine>, String> {
    let output = ProcessCommand::new("git")
        .args([
            "blame",
            "--line-porcelain",
            commit_hash,
            "--",
            file_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run git blame: {}", e))?;

    if !output.status.success() {
        return Err("Git blame failed".to_string());
    }

    let output_str = std::str::from_utf8(&output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in git blame output: {}", e))?;

    Ok(parse_blame_output_with_highlighting(output_str, file_path))
}

fn parse_blame_output_with_highlighting(blame_text: &str, file_path: &str) -> Vec<BlameLine> {
    // Load syntax and theme sets once for the entire file
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    
    // Try to determine syntax by file extension
    let syntax = ps.find_syntax_for_file(file_path)
        .unwrap_or(None)
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    
    // Use a dark theme
    let theme = &ts.themes["base16-ocean.dark"];
    let mut h = HighlightLines::new(syntax, theme);
    
    let mut blame_lines = Vec::new();
    let lines: Vec<&str> = blame_text.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        if let Some(line) = lines.get(i) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0].len() >= 7 {
                let commit_hash = parts[0].to_string();
                let line_number: usize = parts[2].parse().unwrap_or(0);
                
                // Look for author, date, and summary in the following lines
                let mut author = String::new();
                let mut date = String::new();
                let mut commit_message = String::new();
                let mut content = String::new();
                
                i += 1;
                while i < lines.len() {
                    if let Some(info_line) = lines.get(i) {
                        if info_line.starts_with("author ") {
                            author = info_line[7..].to_string();
                        } else if info_line.starts_with("author-time ") {
                            // Convert timestamp to readable date
                            if let Ok(timestamp) = info_line[12..].parse::<i64>() {
                                date = format_timestamp(timestamp);
                            }
                        } else if info_line.starts_with("summary ") {
                            commit_message = info_line[8..].to_string();
                        } else if info_line.starts_with('\t') {
                            content = info_line[1..].to_string(); // Remove leading tab
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
                
                // Apply syntax highlighting to this line
                let highlighted_content = if content.len() > 200 {
                    // For very long lines, skip highlighting for performance
                    content.clone()
                } else {
                    let ranges: Vec<(Style, &str)> = h.highlight_line(&content, &ps).unwrap_or_default();
                    as_24_bit_terminal_escaped(&ranges[..], false)
                };
                
                blame_lines.push(BlameLine {
                    line_number,
                    author: abbreviate_author(&author),
                    date,
                    commit_hash: commit_hash[..7].to_string(),
                    commit_message,
                    content,
                    highlighted_content,
                });
            } else {
                i += 1;
            }
        } else {
            break;
        }
    }
    
    blame_lines
}

fn run_interactive_viewer(file_path: &str, versions: Vec<FileVersion>, start_line: usize, end_line: usize) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    let mut current_version = 0;
    let mut scroll_offset = 0;
    let mut target_line: Option<usize> = None; // Track the line we're trying to stay close to

    loop {
        let (terminal_width, terminal_height) = crossterm::terminal::size()?;
        let content_height = terminal_height as usize - 4; // Reserve space for 3-line header + 1-line footer

        // Clear screen and draw content
        execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

        // Header with colors
        let version = &versions[current_version];
        execute!(stdout, SetForegroundColor(Color::White), SetBackgroundColor(Color::DarkBlue))?;

        // Main header line with file, version number, and date
        let header_text = format!("📜 {} │ {} of {} │ 📅 {}",
            file_path,
            current_version + 1,
            versions.len(),
            version.commit_date
        );
        print!("{}", header_text);

        // Pad to full width
        if header_text.len() < terminal_width as usize {
            print!("{}", " ".repeat(terminal_width as usize - header_text.len()));
        }
        execute!(stdout, ResetColor)?;
        println!("\r");

        // Commit details line
        execute!(stdout, SetForegroundColor(Color::Yellow))?;
        let commit_short = if version.commit_hash.len() > 8 {
            &version.commit_hash[..8]
        } else {
            &version.commit_hash
        };
        let commit_line = format!("🔗 {} │ {}", commit_short, version.commit_message);

        // Truncate commit message if too long
        let max_commit_line_len = terminal_width as usize;
        let display_commit_line = if commit_line.len() > max_commit_line_len {
            format!("{}...", &commit_line[..max_commit_line_len.saturating_sub(3)])
        } else {
            commit_line
        };

        print!("{}", display_commit_line);
        if display_commit_line.len() < terminal_width as usize {
            print!("{}", " ".repeat(terminal_width as usize - display_commit_line.len()));
        }
        execute!(stdout, ResetColor)?;
        println!("\r");

        // Separator line
        execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
        println!("{}\r", "─".repeat(terminal_width as usize));
        execute!(stdout, ResetColor)?;

        // Content with colors (filtered by line range)
        // Adjust end_line if it exceeds the available lines in this version
        let max_line_in_version = version.blame_lines.iter()
            .map(|line| line.line_number)
            .max()
            .unwrap_or(0);

        let adjusted_end_line = if end_line > max_line_in_version {
            max_line_in_version
        } else {
            end_line
        };

        let filtered_lines: Vec<&BlameLine> = version.blame_lines.iter()
            .filter(|line| line.line_number >= start_line && line.line_number <= adjusted_end_line)
            .collect();

        // If we have a target line, try to position the view around it
        if let Some(target) = target_line {
            if let Some(closest_pos) = find_closest_line_in_filtered(&filtered_lines, target) {
                // Center the view around the closest line
                scroll_offset = closest_pos.saturating_sub(content_height / 2);
                if scroll_offset + content_height > filtered_lines.len() {
                    scroll_offset = filtered_lines.len().saturating_sub(content_height);
                }
            }
        }
        
        let display_end = (scroll_offset + content_height).min(filtered_lines.len());
        let mut last_author = String::new();
        let content_width = terminal_width as usize - 20; // Reserve space for line numbers and margins
        
        for i in scroll_offset..display_end {
            if let Some(line) = filtered_lines.get(i) {
                // Check if we need to show author info (first line or author changed)
                let show_author = last_author != line.author;
                if show_author {
                    last_author = line.author.clone();
                    
                    // Author header line with color
                    let author_color = get_author_color(&line.author);
                    execute!(stdout, SetForegroundColor(author_color))?;
                    print!("┌─ {} ", line.author);
                    execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
                    print!("({}) ", line.date);
                    execute!(stdout, SetForegroundColor(Color::Yellow))?;
                    print!("[{}] ", line.commit_hash);
                    execute!(stdout, SetForegroundColor(Color::White))?;
                    print!("{}", line.commit_message);
                    execute!(stdout, ResetColor)?;
                    println!("\r");
                }
                
                // Line number with proper spacing
                execute!(stdout, SetForegroundColor(Color::DarkGrey))?;
                if show_author {
                    print!("│ {:3} │ ", line.line_number);
                } else {
                    print!("│ {:3} │ ", line.line_number);
                }
                execute!(stdout, ResetColor)?;
                
                // Content with line wrapping - use pre-rendered highlighted content
                let content = &line.content;
                let highlighted_content = &line.highlighted_content;
                
                if content.len() <= content_width {
                    // Single line - no wrapping needed
                    println!("{}\r", highlighted_content);
                } else {
                    // For long lines, just truncate to avoid wrapping complexity with ANSI codes
                    if content.len() > content_width {
                        // Use plain content for truncation to avoid cutting ANSI escape sequences
                        let truncated = format!("{}...", &content[..content_width.saturating_sub(3)]);
                        println!("{}\r", truncated);
                    } else {
                        println!("{}\r", highlighted_content);
                    }
                }
            }
        }
        // Footer with colors
        execute!(stdout, crossterm::cursor::MoveTo(0, terminal_height - 1))?;
        execute!(stdout, SetForegroundColor(Color::White), SetBackgroundColor(Color::DarkGrey))?;
        print!("← Older    Newer → │ ↑ ↓ : Scroll │ Mouse: Scroll │ q : Quit");
        // Pad footer to full width
        let footer_text = "← Older    Newer → │ ↑ ↓ : Scroll │ Mouse: Scroll │ q : Quit";
        if footer_text.len() < terminal_width as usize {
            print!("{}", " ".repeat(terminal_width as usize - footer_text.len()));
        }
        execute!(stdout, ResetColor)?;
        print!("\r");
        stdout.flush()?;
        // Handle input including mouse
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Left => {
                            if current_version > 0 {
                                // Capture current target line before switching
                                target_line = Some(get_current_target_line(&filtered_lines, scroll_offset, content_height));
                                current_version -= 1;
                            }
                        }
                        KeyCode::Right => {
                            if current_version < versions.len() - 1 {
                                // Capture current target line before switching
                                target_line = Some(get_current_target_line(&filtered_lines, scroll_offset, content_height));
                                current_version += 1;
                            }
                        }
                        KeyCode::Up => {
                            if scroll_offset > 0 {
                                scroll_offset -= 1;
                                target_line = None; // Clear target line on manual scroll
                            }
                        }
                        KeyCode::Down => {
                            if scroll_offset + content_height < filtered_lines.len() {
                                scroll_offset += 1;
                                target_line = None; // Clear target line on manual scroll
                            }
                        }
                        KeyCode::PageUp => {
                            scroll_offset = scroll_offset.saturating_sub(content_height / 2);
                            target_line = None; // Clear target line on manual scroll
                        }
                        KeyCode::PageDown => {
                            scroll_offset = (scroll_offset + content_height / 2).min(filtered_lines.len().saturating_sub(content_height));
                            target_line = None; // Clear target line on manual scroll
                        }
                        KeyCode::Home => {
                            scroll_offset = 0;
                            target_line = None; // Clear target line on manual scroll
                        }
                        KeyCode::End => {
                            scroll_offset = filtered_lines.len().saturating_sub(content_height);
                            target_line = None; // Clear target line on manual scroll
                        }
                        _ => {}
                    }
                }
            }
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        scroll_offset = scroll_offset.saturating_sub(3); // Scroll 3 lines at a time
                        target_line = None; // Clear target line on manual scroll
                    }
                    MouseEventKind::ScrollDown => {
                        let max_scroll = if filtered_lines.len() > content_height {
                            filtered_lines.len() - content_height
                        } else {
                            0
                        };
                        scroll_offset = (scroll_offset + 3).min(max_scroll);
                        target_line = None; // Clear target line on manual scroll
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    // Cleanup
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

fn get_commit_changes(commit_hash: &str, file_path: &str, start_line: usize, end_line: usize) -> Result<Vec<LineChange>, String> {
    let range = format!("{},{}", start_line, end_line);
    let output = ProcessCommand::new("git")
        .args([
            "show",
            commit_hash,
            "-L", &format!("{}:{}", range, file_path),
        ])
        .output()
        .map_err(|e| format!("Failed to run git show: {}", e))?;

    if !output.status.success() {
        return Ok(vec![]); // Return empty if git show fails
    }

    let output_str = std::str::from_utf8(&output.stdout)
        .map_err(|e| format!("Invalid UTF-8 in git show output: {}", e))?;

    Ok(parse_diff_output(output_str))
}

fn parse_diff_output(diff_text: &str) -> Vec<LineChange> {
    let mut changes = Vec::new();
    let mut in_diff = false;
    let mut line_number = 0;

    for line in diff_text.lines() {
        // Look for the @@ hunk header to start parsing
        if line.starts_with("@@") {
            in_diff = true;
            // Parse the line number from @@ -old_start,old_count +new_start,new_count @@
            if let Some(plus_pos) = line.find('+') {
                if let Some(comma_pos) = line[plus_pos..].find(',') {
                    let start_str = &line[plus_pos + 1..plus_pos + comma_pos];
                    line_number = start_str.parse().unwrap_or(1);
                } else if let Some(space_pos) = line[plus_pos..].find(' ') {
                    let start_str = &line[plus_pos + 1..plus_pos + space_pos];
                    line_number = start_str.parse().unwrap_or(1);
                }
            }
            continue;
        }

        if !in_diff {
            continue;
        }

        // Stop at the next commit or end of diff
        if line.starts_with("commit ") || line.starts_with("diff --git") {
            break;
        }

        if line.starts_with('+') && !line.starts_with("+++") {
            changes.push(LineChange {
                line_number,
                change_type: ChangeType::Added,
                content: line[1..].to_string(), // Remove the + prefix
            });
            line_number += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            changes.push(LineChange {
                line_number,
                change_type: ChangeType::Removed,
                content: line[1..].to_string(), // Remove the - prefix
            });
            // Don't increment line_number for removed lines
        } else if line.starts_with(' ') {
            // Context line - increment line number but don't show it
            line_number += 1;
        }
    }

    changes
}

fn display_change(change: &LineChange) {
    let prefix = match change.change_type { ChangeType::Added => "│  +", ChangeType::Removed => "│  -",
        ChangeType::Modified => "│  ~",
    };
    
    println!("{} {}", prefix, change.content);
}