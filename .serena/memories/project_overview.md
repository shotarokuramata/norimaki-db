# Norimaki-DB Project Overview

## Purpose
Norimaki-DB is a simple key-value store designed for learning database concepts. It's a Rust library that provides both in-memory and file-based storage implementations.

## Tech Stack
- **Language**: Rust (Edition 2021)
- **Dependencies**: 
  - serde 1.0 (with derive feature) - for serialization/deserialization
  - serde_json 1.0 - for JSON serialization
- **Build System**: Cargo

## Key Features
- Two storage implementations:
  - `MemoryStore` - In-memory HashMap-based storage
  - `FileStore` - Persistent JSON file-based storage
- Common `KeyValueStore` trait for both implementations
- Basic operations: put, get, delete, keys, clear
- Error handling with custom `StoreError` enum

## Project Structure
```
norimaki-db/
├── Cargo.toml          # Project configuration and dependencies
├── Cargo.lock          # Dependency lock file
├── .gitignore          # Git ignore rules (/target)
├── test_db.json        # Test database file (untracked)
└── src/
    ├── lib.rs          # Main library file with tests
    ├── store.rs        # Storage implementations
    └── error.rs        # Error types and handling
```

## Storage Implementations
1. **MemoryStore**: Fast, non-persistent HashMap-based storage
2. **FileStore**: Persistent storage using JSON files with automatic save/load