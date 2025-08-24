# garch - Git Archaeology

A command-line tool that transforms git history into an explorable time dimension. Instead of viewing code as static text, `garch` lets you navigate through every iteration and understand the evolution of your codebase.

## What is Git Archaeology?

Most files exist only in the present moment - you can't step back through time to see how they evolved. But git gives code an extra dimension: a traversable timeline of every change. `garch` makes this timeline accessible and intuitive to explore.

## Installation

### From Source

```bash
git clone https://github.com/yourusername/garch
cd garch
cargo install --path .
```

### Prerequisites

- Rust 1.70 or later
- Git (any recent version)

## Usage

### Trace Line Evolution

See how specific lines evolved through history:

```bash
# Trace lines 10-20 in a file
garch lines src/auth.rs:10-20

# Trace a single line
garch lines src/main.rs:45

# Trace an entire function (specify the range)
garch lines lib/utils.py:120-150
```

Output shows a timeline of changes:

```
📜 Evolution of src/auth.rs:10-20

├─ Commit abc1234 (2024-01-15) - Alice C.
│  "Add basic password validation"
│  + def validate_password(password):
│  +     if len(password) < 8:
│  +         return False
│  +     return True
│
└─ Commit def5678 (2024-02-03) - Bob S.
   "Strengthen password requirements" 
   ~ def validate_password(password):
   ~     if len(password) < 12:  # was 8
   ~         return False
   +     if not any(c.isupper() for c in password):
   +         return False
   ~     return True
```

### Interactive File Explorer

Navigate through different versions of a file:

```bash
garch file src/main.rs
```

This launches an interactive terminal interface where you can:

- **← →** Navigate between different commits of the file
- **↑ ↓** Scroll through the current file version  
- **Page Up/Down** Jump larger chunks through the file
- **Mouse wheel** Scroll (3 lines at a time)
- **Home/End** Jump to top/bottom of file
- **q** Quit

The interface shows each line with:
- Line number
- Author (color-coded for easy recognition)
- Last modification date
- Line content

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
# You found a bug on line 127 of user_service.py
garch lines src/user_service.py:120-135

# This shows you:
# - When the problematic code was introduced
# - Who wrote it and why (commit messages)
# - How it evolved over time
# - What the original implementation looked like
```

### Understanding Complex Code

```bash
# You're looking at a confusing function
garch file src/complex_algorithm.py

# Navigate through the file's history to see:
# - How it started (probably simple)
# - What requirements forced complexity
# - Who worked on different parts
# - The reasoning behind each change
```

### Code Review Context

```bash
# Before reviewing a change to authentication logic
garch lines src/auth/validator.rs:50-80

# This gives you context:
# - Previous approaches that were tried
# - Why certain decisions were made
# - Potential edge cases from history
# - Who has domain knowledge
```

## Technical Details

- **Language**: Rust (for performance and reliability)
- **Terminal UI**: crossterm (cross-platform terminal control)
- **Git Integration**: Shell commands (maximum compatibility)
- **Performance**: Git operations are cached, UI rendering is optimized

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

- [ ] Syntax highlighting for different file types
- [ ] Search within file versions
- [ ] Export functionality (HTML reports)
- [ ] Support for comparing two arbitrary versions
- [ ] Integration with external diff tools
- [ ] Performance optimizations for very large repositories

## License

MIT License - see LICENSE file for details

## Why "garch"?

Git + Archaeology = garch. It's short, memorable, and captures the essence of digging through code history to understand the present.