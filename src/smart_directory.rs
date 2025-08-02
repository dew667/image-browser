use chrono::prelude::*;
use serde;
use serde_json;
use std::fs;
use std::{error::Error, path::PathBuf};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct RecentItem {
    path: PathBuf,
    last_viewed: DateTime<Local>,
    view_count: u32,
    file_size: u64,
    last_modified: DateTime<Local>,
}

impl RecentItem {
    pub fn new(path: PathBuf) -> Self {
        let file_size = path.metadata().map(|meta| meta.len()).unwrap_or(0);
        RecentItem {
            path,
            last_viewed: Local::now(),
            view_count: 1,
            file_size: file_size,
            last_modified: Local::now(),
        }
    }

    pub fn name(&self) -> String {
        self.path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct RecentManager {
    recent_items: Vec<RecentItem>,
    max_items: usize,
}

impl RecentManager {
    pub fn new(max_items: usize) -> Self {
        RecentManager {
            recent_items: Vec::new(),
            max_items,
        }
    }

    pub fn add_item(&mut self, path: PathBuf) {
        if let Some(index) = self.recent_items.iter().position(|item| item.path == path) {
            self.recent_items[index].view_count += 1;
            self.recent_items[index].last_viewed = Local::now();
            self.recent_items[index].last_modified = Local::now();
        } else {
            self.recent_items.push(RecentItem::new(path));
            if self.recent_items.len() > self.max_items {
                self.recent_items.remove(0);
            }
        }
    }

    pub fn get_recent_items(&self) -> &[RecentItem] {
        &self.recent_items
    }

    pub fn load_from_file(path: PathBuf) -> Result<RecentManager, Box<dyn Error>> {
        if !path.is_file() {
            return Ok(RecentManager::new(10));
        }
        let content = fs::read_to_string(path)?;
        let manager: RecentManager = serde_json::from_str(&content)?;
        Ok(manager)
    }

    pub fn save_to_file(&self, path: PathBuf) -> Result<(), Box<dyn Error>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
