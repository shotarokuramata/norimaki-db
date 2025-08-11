# Suggested Commands

## Development Commands

### Building and Testing
- `cargo build` - Build the project
- `cargo test` - Run all tests
- `cargo check` - Quick compile check without building
- `cargo clippy` - Run Rust linter
- `cargo fmt` - Format code according to Rust standards

### Running and Documentation
- `cargo doc` - Generate documentation
- `cargo doc --open` - Generate and open documentation in browser

### Package Management
- `cargo update` - Update dependencies
- `cargo tree` - Show dependency tree

## System Commands (Linux)
- `ls` - List files and directories
- `cd` - Change directory
- `grep` - Search text in files
- `find` - Find files and directories
- `git` - Git version control commands

## Project-Specific Notes
- The project creates test files like `test_db.json` during testing
- Test files are automatically cleaned up by tests
- No specific run commands as this is a library crate, not a binary