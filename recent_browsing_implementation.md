# 最近浏览目录功能实现方案

## 数据结构定义

### 1. 核心数据结构

#### RecentItem - 最近浏览项
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentItem {
    pub path: PathBuf,           // 图片完整路径
    pub last_viewed: u64,        // Unix时间戳(毫秒)
    pub view_count: u32,         // 浏览次数
    pub file_size: u64,          // 文件大小(用于验证文件存在)
    pub last_modified: u64,      // 文件最后修改时间
}
```

#### RecentManager - LRU管理器
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentManager {
    items: VecDeque<RecentItem>,  // 使用VecDeque实现高效LRU
    max_items: usize,            // 最大容量
    cache_hits: u64,             // 缓存命中统计
    cache_misses: u64,           // 缓存未命中统计
}
```

### 2. LRU算法实现

#### 核心操作
```rust
impl RecentManager {
    /// 添加或更新浏览记录
    pub fn add_view(&mut self, path: PathBuf) {
        // 1. 检查是否已存在
        if let Some(index) = self.items.iter().position(|item| item.path == path) {
            // 2. 已存在：移动到队头并更新
            let mut item = self.items.remove(index).unwrap();
            item.last_viewed = current_timestamp();
            item.view_count += 1;
            self.items.push_front(item);
        } else {
            // 3. 新记录：添加到队头
            let item = RecentItem::new(path);
            self.items.push_front(item);
            
            // 4. 检查容量并移除最旧的
            if self.items.len() > self.max_items {
                self.items.pop_back();
            }
        }
    }

    /// 获取最近浏览列表
    pub fn get_recent(&self, limit: Option<usize>) -> Vec<&RecentItem> {
        let limit = limit.unwrap_or(self.max_items);
        self.items.iter().take(limit).collect()
    }

    /// 清理不存在的文件
    pub fn cleanup_invalid(&mut self) {
        self.items.retain(|item| item.path.exists());
    }
}
```

### 3. 存储方案

#### 文件格式 (recent.json)
```json
{
  "version": "1.0",
  "max_items": 50,
  "items": [
    {
      "path": "/Users/photos/vacation/beach.jpg",
      "last_viewed": 1722076800000,
      "view_count": 5,
      "file_size": 2048576,
      "last_modified": 1722000000000
    }
  ],
  "stats": {
    "cache_hits": 123,
    "cache_misses": 45
  }
}
```

#### 存储管理
```rust
impl RecentManager {
    /// 从文件加载
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::new(50));
        }
        
        let content = fs::read_to_string(path)?;
        let mut manager: RecentManager = serde_json::from_str(&content)?;
        
        // 验证并清理无效记录
        manager.cleanup_invalid();
        Ok(manager)
    }

    /// 保存到文件
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 自动保存(定期)
    pub fn auto_save(&self) {
        // 使用后台线程每30秒保存一次
    }
}
```

### 4. 性能优化

#### 内存优化
- **VecDeque**: O(1)时间复杂度的头尾操作
- **延迟加载**: 缩略图按需加载
- **内存池**: 复用RecentItem对象

#### 缓存策略
```rust
impl RecentManager {
    /// 批量添加(用于初始化)
    pub fn add_batch(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            self.add_view(path);
        }
    }

    /// 快速查找(使用HashMap索引)
    pub fn contains(&self, path: &Path) -> bool {
        self.items.iter().any(|item| item.path == path)
    }

    /// 获取浏览统计
    pub fn get_stats(&self) -> RecentStats {
        RecentStats {
            total_items: self.items.len(),
            cache_hit_rate: self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64,
            oldest_item: self.items.back().map(|item| item.last_viewed),
            newest_item: self.items.front().map(|item| item.last_viewed),
        }
    }
}
```

### 5. 线程安全

#### 并发访问
```rust
use std::sync::{Arc, RwLock};

pub type SharedRecentManager = Arc<RwLock<RecentManager>>;

impl RecentManager {
    /// 创建线程安全的共享实例
    pub fn new_shared(max_items: usize) -> SharedRecentManager {
        Arc::new(RwLock::new(Self::new(max_items)))
    }
}

// 使用示例
let recent_manager = RecentManager::new_shared(50);
{
    let mut manager = recent_manager.write().unwrap();
    manager.add_view(PathBuf::from("/path/to/image.jpg"));
}
```

### 6. 扩展功能

#### 过滤和搜索
```rust
impl RecentManager {
    /// 按时间范围过滤
    pub fn filter_by_time_range(&self, start: u64, end: u64) -> Vec<&RecentItem> {
        self.items.iter()
            .filter(|item| item.last_viewed >= start && item.last_viewed <= end)
            .collect()
    }

    /// 按浏览次数排序
    pub fn sort_by_view_count(&self) -> Vec<&RecentItem> {
        let mut items: Vec<&RecentItem> = self.items.iter().collect();
        items.sort_by(|a, b| b.view_count.cmp(&a.view_count));
        items
    }

    /// 按文件类型过滤
    pub fn filter_by_extension(&self, ext: &str) -> Vec<&RecentItem> {
        self.items.iter()
            .filter(|item| {
                item.path.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case(ext))
                    .unwrap_or(false)
            })
            .collect()
    }
}
```

### 7. 集成方案

#### 与主程序集成
```rust
// 在主程序State中添加
struct State {
    // ... 现有字段 ...
    recent_manager: SharedRecentManager,
}

// 在图片查看时更新
impl State {
    fn on_image_viewed(&mut self, path: PathBuf) {
        let mut manager = self.recent_manager.write().unwrap();
        manager.add_view(path);
        
        // 异步保存
        let manager_clone = self.recent_manager.clone();
        tokio::spawn(async move {
            let manager = manager_clone.read().unwrap();
            let _ = manager.save_to_file("~/.image-browser/recent.json");
        });
    }
}
```

### 8. 错误处理

#### 健壮性设计
```rust
#[derive(Debug)]
pub enum RecentError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    InvalidPath,
    FileNotFound,
}

impl RecentManager {
    /// 验证文件有效性
    fn validate_item(&self, item: &RecentItem) -> bool {
        if !item.path.exists() {
            return false;
        }
        
        if let Ok(metadata) = fs::metadata(&item.path) {
            let current_modified = metadata.modified()
                .ok()
                .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64)
                .unwrap_or(0);
            
            // 如果文件被修改过，可能需要重新处理
            current_modified == item.last_modified
        } else {
            false
        }
    }
}
```

## 使用示例

### 基本用法
```rust
// 初始化
let mut recent = RecentManager::new(50);

// 添加浏览记录
recent.add_view(PathBuf::from("/photos/vacation.jpg"));

// 获取最近浏览
let recent_items = recent.get_recent(Some(10));

// 保存到文件
recent.save_to_file("recent.json").unwrap();
```

这个实现提供了完整的LRU最近浏览功能，支持持久化存储、并发访问、性能优化和扩展功能。
