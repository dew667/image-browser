use std::path::PathBuf;
use chrono::prelude::*;

pub struct RecentItem {
    path: PathBuf,
    last_viewed: DateTime<Local>,
    view_count: u32,
    file_size: u64,
    last_modified: DateTime<Local>
}

impl RecentItem {
    pub fn new(path: PathBuf) -> Self {
        let file_size = path.metadata()
            .map(|meta| meta.len())
            .unwrap_or(0);
        RecentItem {
            path,
            last_viewed: Local::now(),
            view_count: 1,
            file_size: file_size,
            last_modified: Local::now(),
        }
    }

    pub fn name(&self) -> String {
        self.path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }
}

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
        
    }
}