# Coding Style and Conventions

## Rust Conventions
- Uses Rust 2021 edition
- Standard Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Follows Rust best practices for error handling with custom Result types

## Code Style Patterns
- **Traits**: Uses trait-based design with `KeyValueStore` trait for polymorphism
- **Error Handling**: Custom error enum `StoreError` with proper Display and Error trait implementations
- **Serialization**: Uses serde for JSON serialization with `#[derive(Serialize, Deserialize)]`
- **Memory Management**: Uses owned `String` types for keys and values
- **Validation**: Validates empty keys and returns appropriate errors

## File Organization
- Modular structure with separate files for different concerns:
  - `lib.rs` - Main library interface and integration tests
  - `store.rs` - Storage trait and implementations
  - `error.rs` - Error types and conversions
- Public API exposed through re-exports in lib.rs

## Testing Patterns
- Integration tests in lib.rs using `#[cfg(test)]`
- Test functions use descriptive names like `test_memory_store_basic_operations`
- Tests clean up after themselves (removing test files)
- Tests both successful operations and error conditions

## Documentation
- No extensive documentation comments found - follows "code is documentation" approach
- Descriptive function and variable names