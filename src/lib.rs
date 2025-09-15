//! # Norimaki DB
//! 
//! A high-performance key-value store optimized for boat racing data management.
//!
//! ## Quick Start
//!
//! ```rust
//! use norimaki_db::{BoatRaceEngine, MemoryStore, MonthlySchedule, RaceEvent};
//!
//! // Create engine with in-memory storage
//! let store = MemoryStore::new();
//! let mut engine = BoatRaceEngine::new(store);
//!
//! // Save monthly schedule
//! let schedule = MonthlySchedule {
//!     year_month: "2025-09".to_string(),
//!     events: vec![/* ... */],
//! };
//! engine.put_monthly_schedule(&schedule)?;
//!
//! // Retrieve monthly schedule
//! let retrieved = engine.get_monthly_schedule(202509)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod error;
pub mod store;
pub mod key;
pub mod value;
pub mod engine;

// Core types and results
pub use error::{Result, StoreError};

// Storage backends
pub use store::{FileStore, KeyValueStore, MemoryStore};

// Main engine
pub use engine::BoatRaceEngine;

// Key generation utilities (commonly used)
pub use key::{generate_tournament_id, monthly_key, tournament_key};

// Serialization utilities (for custom data types)
pub use value::{serialize_to_string, deserialize_from_string};

// Re-export commonly used types from dependencies
pub use serde::{Serialize, Deserialize};

/// Monthly schedule containing a list of race events for a specific month
/// 
/// # Example
/// ```rust
/// use norimaki_db::{MonthlySchedule, RaceEvent};
/// 
/// let schedule = MonthlySchedule {
///     year_month: "2025-09".to_string(),
///     events: vec![/* RaceEvent instances */],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySchedule {
    /// Year and month in "YYYY-MM" format (e.g., "2025-09")
    pub year_month: String,
    /// List of race events in this month
    pub events: Vec<RaceEvent>,
}

/// Information about a single race event/tournament
/// 
/// # Example
/// ```rust
/// use norimaki_db::RaceEvent;
/// 
/// let event = RaceEvent {
///     venue_id: 4,
///     venue_name: "平和島".to_string(),
///     event_name: "トーキョー・ベイ・カップ".to_string(),
///     grade: "G1".to_string(),
///     start_date: "2025-09-10".to_string(),
///     duration_days: 7,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaceEvent {
    /// Unique venue identifier
    pub venue_id: u32,
    /// Name of the racing venue
    pub venue_name: String,
    /// Name of the event/tournament
    pub event_name: String,
    /// Grade of the event (e.g., "G1", "G2", "一般", "SG")
    pub grade: String,
    /// Start date in "YYYY-MM-DD" format
    pub start_date: String,
    /// Duration of the event in days
    pub duration_days: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_memory_store_basic_operations() {
        let mut store = MemoryStore::new();

        assert!(store.put("key1".to_string(), "value1".to_string()).is_ok());

        let result = store.get("key1").unwrap();
        assert_eq!(result, Some("value1".to_string()));

        assert!(store.delete("key1").is_ok());
        let result = store.get("key1").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_memory_store_invalid_key() {
        let mut store = MemoryStore::new();

        assert!(store.put("".to_string(), "value".to_string()).is_err());
        assert!(store.get("").is_err());
        assert!(store.delete("").is_err());
    }

    #[test]
    fn test_memory_store_keys_and_clear() {
        let mut store = MemoryStore::new();

        store.put("key1".to_string(), "value1".to_string()).unwrap();
        store.put("key2".to_string(), "value2".to_string()).unwrap();

        let keys = store.keys().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));

        store.clear().unwrap();
        assert_eq!(store.keys().unwrap().len(), 0);
    }

    #[test]
    fn test_file_store_basic_operations() {
        let test_file = "test_db.json";

        {
            let mut store = FileStore::new(test_file).unwrap();

            assert!(store.put("key1".to_string(), "value1".to_string()).is_ok());

            let result = store.get("key1").unwrap();
            assert_eq!(result, Some("value1".to_string()));
        }

        {
            let store = FileStore::new(test_file).unwrap();
            let result = store.get("key1").unwrap();
            assert_eq!(result, Some("value1".to_string()));
        }

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_file_store_persistence() {
        let test_file = "test_persistence.json";

        {
            let mut store = FileStore::new(test_file).unwrap();
            store
                .put("persistent_key".to_string(), "persistent_value".to_string())
                .unwrap();
        }

        {
            let mut store = FileStore::new(test_file).unwrap();
            let result = store.get("persistent_key").unwrap();
            assert_eq!(result, Some("persistent_value".to_string()));

            store.delete("persistent_key").unwrap();
            assert_eq!(store.get("persistent_key").unwrap(), None);
        }

        fs::remove_file(test_file).ok();
    }

    // テストデータをinclude!で読み込み
    include!("../testdata/sample.rs");

    #[test]
    fn test_memory_store_scan_with_sample_data() {
        let mut store = MemoryStore::new();
        let data = sample_data();

        // サンプルデータから月別ビューキーを生成してストアに挿入
        for event in &data.events {
            let key = format!("M202509\x00{}", event.venue_name);
            let value = format!("{}-{}", event.event_name, event.grade);
            store.put(key, value).unwrap();
        }

        // 追加データ（範囲外）
        store.put("M202510\x00autumn_cup".to_string(), "秋季大会-G2".to_string()).unwrap();

        // 2025年9月のスキャンテスト
        let results = store.scan("M202509", "M202510").unwrap();
        assert_eq!(results.len(), 3); // sample.rsには3つのイベントがある

        // 結果の検証
        let values: Vec<String> = results.iter().map(|(_, v)| v.clone()).collect();
        assert!(values.iter().any(|v| v.contains("群馬クレインサンダーズカップ")));
        assert!(values.iter().any(|v| v.contains("トーキョー・ベイ・カップ")));
        assert!(values.iter().any(|v| v.contains("高松宮記念")));
        
        // 範囲外のデータが含まれないことを確認
        assert!(!values.iter().any(|v| v.contains("秋季大会")));
    }

    #[test]
    fn test_file_store_scan_with_sample_data() {
        let test_file = "test_scan_sample.json";
        let data = sample_data();

        {
            let mut store = FileStore::new(test_file).unwrap();

            // サンプルデータを挿入
            for event in &data.events {
                let key = format!("M202509\x00{}", event.venue_name);
                let value = serde_json::to_string(event).unwrap();
                store.put(key, value).unwrap();
            }

            // スキャンテスト
            let results = store.scan("M202509", "M202510").unwrap();
            assert_eq!(results.len(), 3);
            
            // G1グレードの大会が2つあることを確認
            let g1_count = results.iter()
                .filter(|(_, v)| v.contains("\"G1\""))
                .count();
            assert_eq!(g1_count, 2);
        }

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_scan_invalid_keys() {
        let mut store = MemoryStore::new();

        // 空文字列でのスキャンはエラー
        assert!(store.scan("", "end").is_err());
        assert!(store.scan("start", "").is_err());
        assert!(store.scan("", "").is_err());
    }
}
