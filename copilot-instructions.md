# Copilot Instructions for garch (Git Archaeology)

## Project Overview
`garch` is a Rust command-line tool that makes git history explorable as a navigable time dimension. It transforms static code into an interactive timeline where developers can trace the evolution of any line or file through its complete history.

## Core Architecture

### Key Components
- **CLI Interface**: Uses `clap` for command parsing with two main subcommands:
  - `garch lines file.rs:10-20` - Trace specific line ranges
  - `garch file src/main.rs` - Interactive file explorer
- **Git Integration**: Shells out to git commands rather than using libgit2
  - `git log -L` for line history
  - `git blame --line-porcelain` for authorship data
  - `git show` for diff parsing
- **Terminal UI**: Uses `crossterm` for interactive terminal interface with colors and mouse support

### Data Structures
```rust
struct CommitInfo { hash, date, author, message }
struct BlameLine { line_number, author, date, commit_hash, content }
struct FileVersion { commit_hash, commit_date, commit_message, blame_lines }
struct LineChange { line_number, change_type, content }
```

## Development Guidelines

### Git Command Usage
- Always use existing git plumbing commands instead of implementing git parsing
- Handle git command failures gracefully - some commits may not have blame data
- Use `--line-porcelain` for blame to get structured output
- Use `git log --follow` to track files through renames

### Performance Considerations
- Git operations are the bottleneck, not Rust code
- Cache blame data when navigating between file versions
- Limit concurrent git processes to avoid overwhelming the system
- Consider using `git cat-file --batch` for bulk operations

### Terminal UI Patterns
- Always clean up terminal state (disable raw mode, leave alternate screen)
- Handle terminal resize events gracefully
- Use consistent color schemes - hash author names for consistent colors
- Implement smooth scrolling (3 lines at a time for mouse wheel)
- Provide clear navigation hints in footer

### Error Handling
- Git commands can fail for many reasons - always provide fallbacks
- Handle UTF-8 conversion errors from git output
- Gracefully handle files that don't exist in certain commits
- Show helpful error messages, not debug output

## Code Style

### Rust Patterns
- Use `Result<T, String>` for git operations that can fail with user-facing errors
- Prefer `std::str::from_utf8` over unwrap for git command output
- Use `saturating_sub` and `min`/`max` for terminal bounds checking
- Keep git parsing functions separate from UI logic

### Git Output Parsing
- Always check for malformed git output before parsing
- Handle edge cases like empty files, binary files, or files with unusual characters
- Use split and slice operations carefully - git output format can vary
- Don't assume line counts or commit hashes will always be present

### UI State Management
- Keep scroll offset and current version as separate concerns
- Reset scroll when changing file versions
- Handle edge cases like empty files or single-commit histories
- Provide visual feedback when operations are in progress

## Testing Approach
- Test with repositories that have complex histories (merges, renames, large files)
- Test edge cases like files with no history, binary files, or files deleted in some commits
- Test terminal behavior on different platforms and terminal emulators
- Verify color output works correctly across different terminal capabilities

## Common Patterns

### Parsing Git Output
```rust
// Always check command success first
if !output.status.success() {
    return Err(format!("Git command failed: {}", 
        std::str::from_utf8(&output.stderr).unwrap_or("unknown error")));
}

// Handle UTF-8 conversion
let output_str = std::str::from_utf8(&output.stdout)
    .map_err(|e| format!("Invalid UTF-8 in git output: {}", e))?;
```

### Terminal Color Management
```rust
// Always reset colors after setting them
execute!(stdout, SetForegroundColor(color))?;
print!("content");
execute!(stdout, ResetColor)?;
```

### Bounds Checking for Scrolling
```rust
// Use saturating arithmetic for scroll operations
scroll_offset = scroll_offset.saturating_sub(amount);
scroll_offset = (scroll_offset + amount).min(max_scroll);
```

## Extension Points
- Add syntax highlighting by detecting file types
- Implement search within file versions
- Add commit message filtering/search
- Support for comparing two arbitrary versions
- Export functionality (HTML, PDF reports)
- Integration with other git tools (diff viewers, merge tools)

## Performance Notes
- Blame operations are expensive - consider caching
- Large files may need pagination or lazy loading
- Terminal rendering is fast, git operations are slow
- Consider parallel git operations for multi-file analysis