use crate::{Result, StoreError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

pub trait KeyValueStore {
    fn put(&mut self, key: String, value: String) -> Result<()>;
    fn get(&self, key: &str) -> Result<Option<String>>;
    fn delete(&mut self, key: &str) -> Result<()>;
    fn keys(&self) -> Result<Vec<String>>;
    fn clear(&mut self) -> Result<()>;
    fn scan(&mut self, start: &str, end: &str) -> Result<Vec<(String, String)>>;
}

#[derive(Debug, Clone)]
pub struct MemoryStore {
    data: HashMap<String, String>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyValueStore for MemoryStore {
    fn put(&mut self, key: String, value: String) -> Result<()> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey);
        }
        self.data.insert(key, value);
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey);
        }
        Ok(self.data.get(key).cloned())
    }

    fn delete(&mut self, key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey);
        }
        self.data.remove(key);
        Ok(())
    }

    fn keys(&self) -> Result<Vec<String>> {
        Ok(self.data.keys().cloned().collect())
    }

    fn clear(&mut self) -> Result<()> {
        self.data.clear();
        Ok(())
    }

    fn scan(&mut self, start: &str, end: &str) -> Result<Vec<(String, String)>> {
        if start.is_empty() || end.is_empty() {
            return Err(StoreError::InvalidKey);
        }
        let mut result = Vec::new();
        for (key, value) in &self.data {
            if key.as_str() >= start && key.as_str() < end {
                result.push((key.clone(), value.clone()));
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FileData {
    data: HashMap<String, String>,
}

#[derive(Debug)]
pub struct FileStore {
    file_path: String,
    data: HashMap<String, String>,
}

impl FileStore {
    pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let file_path = file_path.as_ref().to_string_lossy().to_string();
        let mut store = Self {
            file_path,
            data: HashMap::new(),
        };
        store.load()?;
        Ok(store)
    }

    fn load(&mut self) -> Result<()> {
        if !Path::new(&self.file_path).exists() {
            return Ok(());
        }

        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        if contents.trim().is_empty() {
            return Ok(());
        }

        let file_data: FileData = serde_json::from_str(&contents)?;
        self.data = file_data.data;
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let file_data = FileData {
            data: self.data.clone(),
        };
        let json = serde_json::to_string_pretty(&file_data)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)?;

        file.write_all(json.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }
}

impl KeyValueStore for FileStore {
    fn put(&mut self, key: String, value: String) -> Result<()> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey);
        }
        self.data.insert(key, value);
        self.save()?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey);
        }
        Ok(self.data.get(key).cloned())
    }

    fn delete(&mut self, key: &str) -> Result<()> {
        if key.is_empty() {
            return Err(StoreError::InvalidKey);
        }
        self.data.remove(key);
        self.save()?;
        Ok(())
    }

    fn keys(&self) -> Result<Vec<String>> {
        Ok(self.data.keys().cloned().collect())
    }

    fn clear(&mut self) -> Result<()> {
        self.data.clear();
        self.save()?;
        Ok(())
    }

    fn scan(&mut self, start: &str, end: &str) -> Result<Vec<(String, String)>> {
        if start.is_empty() || end.is_empty() {
            return Err(StoreError::InvalidKey);
        }
        let mut result = Vec::new();
        for (key, value) in &self.data {
            if key.as_str() >= start && key.as_str() < end {
                result.push((key.clone(), value.clone()));
            }
        }
        Ok(result)
    }
}
