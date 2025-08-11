# Task Completion Checklist

## When Completing Tasks in Norimaki-DB

### 1. Code Quality Checks
- [ ] Run `cargo check` - Ensure code compiles without errors
- [ ] Run `cargo clippy` - Check for Rust best practices and potential issues
- [ ] Run `cargo fmt` - Format code according to Rust standards

### 2. Testing
- [ ] Run `cargo test` - Ensure all tests pass
- [ ] Add tests for new functionality if applicable
- [ ] Verify tests clean up any temporary files they create

### 3. Documentation
- [ ] Update code comments if adding public APIs
- [ ] Run `cargo doc` to ensure documentation builds without warnings

### 4. Build Verification
- [ ] Run `cargo build` - Ensure clean build
- [ ] Check for any compiler warnings and address them

### 5. Git and Version Control
- [ ] Only commit when explicitly requested by user
- [ ] Clean up any test artifacts before committing (test_db.json, etc.)

## Common Commands to Run After Changes
```bash
cargo fmt && cargo clippy && cargo test && cargo build
```

## Files to Watch
- Test files like `test_db.json` are created during testing but should not be committed
- Target directory (`/target`) is ignored by git