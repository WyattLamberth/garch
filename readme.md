# garch - Git Archaeology

A command-line tool for exploring git history as an interactive timeline. Navigate through every version of your code to understand how it evolved, who wrote what, and why changes were made.

## What is Git Archaeology?

Most files exist only in the present moment - you can't step back through time to see how they evolved. But git gives code an extra dimension: a traversable timeline of every change. `garch` makes this timeline accessible and intuitive to explore with an interactive terminal interface.

## Features

- **🕰️ Interactive Time Navigation**: Step through commits with arrow keys to see how code evolved
- **🎨 Syntax Highlighting**: Full syntax highlighting powered by the same engine used in VS Code
- **👥 Author Tracking**: See who wrote each line, with color-coded author identification
- **📊 Commit Context**: View commit messages, dates, and hashes for full historical context
- **⚡ Performance Optimized**: Pre-rendered syntax highlighting and efficient terminal rendering
- **🖱️ Mouse Support**: Scroll with mouse wheel, navigate with keyboard or mouse
- **🔍 Smart Navigation**: Maintains viewing position when switching between commits

## Installation

### From crates.io (Recommended)

```bash
cargo install garch
```

### From Source

```bash
git clone https://github.com/wyattlamberth/garch
cd garch
cargo install --path .
```

### Prerequisites

- Rust 1.70 or later
- Git (any recent version)

## Usage

### Interactive File Explorer

Navigate through the complete history of a file:

```bash
# View entire file history
garch file src/main.rs

# View file history starting from newest commits
garch file src/auth.rs --reverse
```

### Line Range Analysis

Explore the evolution of specific lines:

```bash
# View evolution of lines 10-20
garch lines src/auth.rs:10-20

# View evolution of a single line
garch lines src/main.rs:45

# View lines starting from newest commits
garch lines lib.py:100-150 --reverse
```

### Command Options

```bash
# Basic commands
garch file <filepath>                    # View entire file history
garch lines <filepath:start-end>         # View specific line range
garch lines <filepath:linenumber>        # View single line

# Options
--reverse, -r                            # Start with newest commits first
--help                                   # Show detailed help
```

### Interactive Controls

The terminal interface provides intuitive navigation:

- **← →** Navigate between different commits (chronological order)
- **↑ ↓** Scroll through the current file version  
- **Page Up/Down** Jump larger chunks through the file
- **Mouse wheel** Scroll (3 lines at a time)
- **Home/End** Jump to top/bottom of file
- **q** Quit

### What You See

Each view shows:
```
filename.rs | 5 of 12 | 2024-01-15 14:30:22
abc1234 | Added error handling and validation

┌─ alice.smith (2024-01-15) [abc1234] Added error handling  
│  45 │ if let Some(error) = result.err() {
│  46 │     log::error!("Processing failed: {}", error);
│  47 │     return Err(error);
│  48 │ }
┌─ bob.jones (2024-01-20) [def5678] Improved error messages
│  49 │ log::info!("Operation completed successfully");
│  50 │ Ok(result)
```

- **File header**: Filename, commit position, and date
- **Commit info**: Hash and commit message
- **Author sections**: Grouped by who wrote the code, with full commit context
- **Line numbers**: Original line numbers from the file
- **Syntax highlighting**: Full color syntax highlighting for the file type

## Use Cases

**Code Archaeology**: Understand why code exists by seeing its evolution and decision points.

**Onboarding**: New team members can trace complex code back to its origins and reasoning.

**Debugging**: Find when bugs were introduced by following the timeline of changes.

**Code Reviews**: Understand the historical context behind current implementations.

**Learning**: Study how codebases evolve and how experienced developers iterate on solutions.

## How It Works

`garch` leverages existing git commands rather than reimplementing git functionality:

- `git log -L` to get commits that touched specific lines
- `git blame --line-porcelain` to get authorship data for each line
- `git show` to extract actual diff content

This approach ensures compatibility with all git repositories and takes advantage of git's optimized history traversal.

## Examples

### Investigating a Bug

```bash
# You found a bug in user_service.py - view the entire file
garch file src/user_service.py

# Or focus on the problematic lines
garch lines src/user_service.py:120-135

# Navigate with ← → to see:
# - When the bug was introduced
# - Who wrote the code and their commit message
# - How the code looked before the bug
# - What changes were made over time
```

### Understanding Complex Code

```bash
# You're looking at a confusing algorithm
garch file src/complex_algorithm.py

# Or focus on the specific complex function
garch lines src/complex_algorithm.py:45-80

# Step through the history to see:
# - How it started (probably much simpler)
# - What requirements drove the complexity
# - Who worked on different sections
# - The evolution of the approach
```

### Code Review Context

```bash
# Before reviewing changes to authentication logic
garch file src/auth/validator.rs

# Or examine specific authentication functions
garch lines src/auth/validator.rs:25-60

# Navigate through commits to understand:
# - Previous approaches that were tried
# - Why certain design decisions were made
# - Historical context for current implementation
# - Who has expertise in this area
```

### Learning from Evolution

```bash
# Study how a configuration file evolved
garch file config/database.yml

# See API endpoint changes over time
garch lines src/api/users.rs:100-200 --reverse

# Understand the progression:
# - Simple initial implementation
# - Performance optimizations added over time
# - Bug fixes and edge case handling
# - Refactoring and architectural changes
```

## Technical Details

- **Language**: Rust (for performance and cross-platform compatibility)
- **Terminal UI**: crossterm (cross-platform terminal control with mouse support)
- **Syntax Highlighting**: syntect (the same highlighting engine used by Sublime Text and VS Code)
- **Git Integration**: Shell commands via `git log -L` and `git blame --line-porcelain` (maximum compatibility)
- **Performance**: Pre-rendered syntax highlighting, efficient screen updates, optimized git operations
- **Platform Support**: Windows, macOS, Linux

### Performance Optimizations

- **Pre-rendered highlighting**: Syntax highlighting is computed once per commit and cached
- **Smart rendering**: Only updates changed screen regions to reduce flicker
- **Efficient scrolling**: Maintains smooth navigation even in large files
- **Git operation caching**: Minimizes repeated git command execution

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

### Development

```bash
# Clone and build
git clone https://github.com/yourusername/garch
cd garch
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- lines src/main.rs:1-10
```

## Roadmap

- [x] ✅ Interactive terminal interface with keyboard/mouse navigation
- [x] ✅ Syntax highlighting for all major programming languages  
- [x] ✅ Author tracking with visual grouping and color coding
- [x] ✅ Commit context display (messages, dates, hashes)
- [x] ✅ Performance optimizations for smooth scrolling
- [ ] 🔄 Search within file versions
- [ ] 🔄 Export functionality (HTML reports, static site generation)
- [ ] 🔄 Side-by-side diff view between any two commits
- [ ] 🔄 Integration with external diff/merge tools
- [ ] 🔄 Git blame integration for line-level commit details
- [ ] 🔄 Support for binary file evolution tracking
- [ ] 🔄 Plugin system for custom file type handling

## License

MIT License - see LICENSE file for details

## Why "garch"?

Git + Archaeology = garch. It's short, memorable, and captures the essence of digging through code history to understand the present.