# Norimaki DB 🚤

A high-performance key-value store optimized for boat racing data management in Rust.

## Features

- **🚀 High Performance**: Optimized 2-tier key design for fast monthly views
- **📅 Time-Series Optimized**: Efficient range queries with timestamp ordering
- **🗓️ Cross-Month Support**: Automatic handling of events spanning multiple months  
- **💾 Multiple Backends**: In-memory (MemoryStore) and persistent (FileStore) storage
- **🔒 Type Safe**: Binary serialization with Rust's type system
- **📊 Built-in Statistics**: Data size and usage monitoring

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
norimaki-db = "0.1.0"
```

### Basic Usage

```rust
use norimaki_db::{BoatRaceEngine, MemoryStore, MonthlySchedule, RaceEvent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine with in-memory storage
    let store = MemoryStore::new();
    let mut engine = BoatRaceEngine::new(store);

    // Create monthly schedule
    let schedule = MonthlySchedule {
        year_month: "2025-09".to_string(),
        events: vec![
            RaceEvent {
                venue_id: 4,
                venue_name: "平和島".to_string(),
                event_name: "トーキョー・ベイ・カップ".to_string(),
                grade: "G1".to_string(),
                start_date: "2025-09-10".to_string(),
                duration_days: 7,
            },
        ],
    };

    // Save and retrieve monthly schedule
    engine.put_monthly_schedule(&schedule)?;
    let retrieved = engine.get_monthly_schedule(202509)?;

    println!("Events in {}: {}", retrieved.year_month, retrieved.events.len());

    Ok(())
}
```

See [examples/quick_start.rs](examples/quick_start.rs) for a complete working example.

### With File Persistence

```rust
use norimaki_db::{BoatRaceEngine, FileStore};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine with file-based storage
    let store = FileStore::new("boat_race_data.json")?;
    let mut engine = BoatRaceEngine::new(store);
    
    // Data is automatically persisted to file
    // ... same usage as above
    
    Ok(())
}
```

### Custom Race Data

You can store any custom data structure that implements `Serialize`/`Deserialize`:

```rust
use norimaki_db::{BoatRaceEngine, MemoryStore, generate_tournament_id};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct RaceData {
    race_number: u32,
    participants: Vec<String>,
    weather: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = BoatRaceEngine::new(MemoryStore::new());

    // Generate unique tournament ID from venue and event name
    let tournament_id = generate_tournament_id("平和島", "トーキョー・ベイ・カップ");

    // Save race data with timestamp
    let race = RaceData {
        race_number: 1,
        participants: vec!["選手A".to_string(), "選手B".to_string()],
        weather: "晴れ".to_string(),
    };

    let timestamp = 1694524800000; // Unix timestamp in milliseconds
    engine.put_race_data(&tournament_id, timestamp, &race)?;

    // Retrieve specific race
    let retrieved_race: RaceData = engine.get_race_data(&tournament_id, timestamp)?;
    println!("Race {}: {} participants", retrieved_race.race_number, retrieved_race.participants.len());

    // Get all races for the tournament
    let all_races: Vec<RaceData> = engine.get_tournament_races(&tournament_id)?;
    println!("Total races: {}", all_races.len());

    Ok(())
}
```

## Architecture

### Key Design

Norimaki DB uses a 2-tier key structure optimized for boat racing data:

```
Monthly View:  M + YYYYMM + 0x00 + tournament_id → RaceEvent (lightweight metadata)
Tournament:    T + tournament_id + 0x00 + timestamp → Race details (full data)
```

This design enables:
- **Fast monthly listing**: Only scans lightweight metadata
- **Efficient race access**: Direct timestamp-based retrieval
- **Cross-month events**: Automatic registration in multiple months

### Data Flow

```
Monthly View (Fast) ────► Tournament Details (On-demand)
     │                         │
     ├─ Event metadata         ├─ Race 1 data
     ├─ Event metadata         ├─ Race 2 data
     └─ Event metadata         └─ Race N data
```

## API Reference

### Core Types

- **`BoatRaceEngine<Store>`**: Main engine for boat racing data operations
- **`MonthlySchedule`**: Contains events for a specific month  
- **`RaceEvent`**: Metadata for a single tournament/event
- **`MemoryStore`**: In-memory storage backend
- **`FileStore`**: File-based persistent storage backend

### Main Operations

- **`put_monthly_schedule(schedule)`**: Save monthly event schedule
- **`get_monthly_schedule(year_month)`**: Retrieve events for a month
- **`put_race_data(tournament_id, timestamp, data)`**: Save race details
- **`get_race_data(tournament_id, timestamp)`**: Retrieve specific race
- **`get_tournament_races(tournament_id)`**: Get all races for a tournament
- **`register_tournament_to_months(event)`**: Handle cross-month events

## Examples

Two examples are provided:

### Quick Start Example
[`examples/quick_start.rs`](examples/quick_start.rs) - Minimal example showing basic usage:
```bash
cargo run --example quick_start
```

### Comprehensive Demo
[`examples/boat_race_demo.rs`](examples/boat_race_demo.rs) - Full-featured demonstration including:
- ✅ Monthly schedule operations
- ✅ File persistence and loading
- ✅ Cross-month tournament handling
- ✅ Custom race data structures
- ✅ Statistics and monitoring

```bash
cargo run --example boat_race_demo
```

## Performance Characteristics

- **Monthly listing**: O(number of events) - independent of race data size
- **Tournament races**: O(number of races in tournament) - single scan
- **Specific race**: O(1) - direct key lookup
- **Cross-month events**: Automatic - no manual management needed

## Use Cases

- **Boat racing management systems**
- **Event scheduling and calendar systems** 
- **Time-series data with hierarchical structure**
- **Any application requiring fast monthly/periodic views**

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Optimized for boat racing data patterns and workflows
- Built with Rust's safety and performance in mind
- Designed for both in-memory and persistent use cases