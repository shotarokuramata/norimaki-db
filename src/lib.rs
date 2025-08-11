pub mod error;
pub mod store;

pub use error::{Result, StoreError};
pub use store::{FileStore, KeyValueStore, MemoryStore};

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
}
