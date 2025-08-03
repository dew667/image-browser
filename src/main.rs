use iced::widget::image::Handle;
use iced::widget::scrollable::Direction;
use iced::widget::{Stack, button, slider};
use iced::{Background, Color, Subscription, Task, Vector, keyboard};
use iced::{
    Element, Length, Theme,
    alignment::Horizontal,
    color,
    widget::{column, container, row, scrollable, text},
};
use image::{GenericImageView, ImageBuffer, ImageReader, Rgb};
use resize::Type::{Catrom, Lanczos3, Mitchell, Point, Triangle};
use rfd::FileDialog;
use rgb::FromSlice;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;

mod button_style;
mod smart_directory;
pub mod cos_client;

use smart_directory::RecentManager;

use crate::cos_client::{CosFunction, TecentCosUtil};
use crate::smart_directory::RecentItem;

// 定义缩放算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResamplingType {
    Point,
    Triangle,
    Catrom,
    Mitchell,
    Lanczos3,
}

impl ResamplingType {
    // 获取算法名称
    fn name(&self) -> &'static str {
        match self {
            ResamplingType::Point => "Point",
            ResamplingType::Triangle => "Triangle",
            ResamplingType::Catrom => "Catrom",
            ResamplingType::Mitchell => "Mitchell",
            ResamplingType::Lanczos3 => "Lanczos3",
        }
    }

    // 获取对应的resize库算法类型
    fn to_resize_type(&self) -> resize::Type {
        match self {
            ResamplingType::Point => Point,
            ResamplingType::Triangle => Triangle,
            ResamplingType::Catrom => Catrom,
            ResamplingType::Mitchell => Mitchell,
            ResamplingType::Lanczos3 => Lanczos3,
        }
    }

    // 获取所有可用算法
    fn all() -> Vec<ResamplingType> {
        vec![
            ResamplingType::Point,
            ResamplingType::Triangle,
            ResamplingType::Catrom,
            ResamplingType::Mitchell,
            ResamplingType::Lanczos3,
        ]
    }
}

struct State {
    current_path: PathBuf,
    current_image: Option<PathBuf>,
    root_file_tree_entry: Vec<FileTreeEntry>,
    image_collection: Vec<PathBuf>, // 用于存储图片库
    current_image_index: usize,
    resampling_bar_opened: bool,       // 是否打开缩放条
    slider_value: u8,                  // 用于缩放条的值
    resampling_type: ResamplingType,   // 当前选择的缩放算法
    original: Option<image::RgbImage>, // 用于存储原始图片
    scaled_bytes: Vec<u8>,             // 用于存储缩放后的图片字节
    thumbnail_cache: std::collections::HashMap<PathBuf, Handle>, // 缓存缩略图
    is_dragging: bool,                 // 是否正在拖动滑块
    last_resize_time: std::time::Instant, // 上次缩放的时间
    preview_scaled_bytes: Vec<u8>,     // 用于存储预览缩放后的图片字节
    final_scaled_bytes: Vec<u8>,       // 用于存储最终高质量缩放后的图片字节
    is_resampling_mode: bool,
    hand_tool_active: bool,                  // 是否启用手型工具
    is_panning: bool,                        // 是否正在拖动画布
    pan_start_position: Option<iced::Point>, // 拖动开始位置
    pan_offset: iced::Vector,                // 拖动偏移量
    recent_manager: RecentManager,
    is_fullscreen: bool,
}

const COLLECTION_LIMIT: usize = 8;

#[derive(Debug, Clone)]
enum Message {
    ChangeDirectory(PathBuf),
    SelectImage,
    NoOp,
    ExpandDirectory(PathBuf),
    PickImage(PathBuf),
    SelectFolder(PathBuf),
    PickNextImage,
    PickPreviousImage,
    OpenResamplingBar,
    SliderChanged(u8),
    SliderReleased,                        // 新增：滑块释放事件
    ResamplingTypeChanged(ResamplingType), // 新增：缩放算法改变
    ImageResized(Vec<u8>, bool),           // 用于接收缩放后的图片字节，bool表示是否是高质量渲染
    LoadImage(PathBuf),                    // 用于加载图片
    LoadThumbnail(PathBuf),                // 用于加载缩略图
    ThumbnailLoaded(PathBuf, Handle),      // 缩略图加载完成
    LoadScaledBytes,                       // 用于加载缩放后的图片字节
    FinalizeDragging,                      // 新增：完成拖动，执行高质量渲染
    ToggleHandTool,                        // 切换手型工具
    MousePressed(iced::mouse::Event),      // 鼠标按下事件
    MouseReleased(iced::mouse::Event),     // 鼠标释放事件
    MouseMoved(iced::Point),               // 鼠标移动事件
    ToggleFullscreen,                      // 切换全屏模式
    EscPressed,                            // ESC按键事件
    UploadToCloud(PathBuf), // 上传到云端
}

#[derive(Debug, Clone)]
enum FileTreeEntry {
    Directory {
        name: String,
        path: PathBuf,
        children: Vec<FileTreeEntry>,
        expanded: bool,
        children_loaded: bool, // 是否已加载子节点
    },
    File {
        name: String,
        path: PathBuf,
    },
}

impl FileTreeEntry {
    fn default(path: PathBuf) -> Self {
        if path.is_dir() {
            FileTreeEntry::Directory {
                name: path.file_name().unwrap().to_string_lossy().into_owned(),
                path,
                children: Vec::new(),
                expanded: false,
                children_loaded: false,
            }
        } else {
            FileTreeEntry::File {
                name: path.file_name().unwrap().to_string_lossy().into_owned(),
                path,
            }
        }
    }

    fn is_directory(&self) -> bool {
        match self {
            FileTreeEntry::Directory { .. } => true,
            FileTreeEntry::File { .. } => false,
        }
    }

    fn path(&self) -> &PathBuf {
        match self {
            FileTreeEntry::Directory { path, .. } => path,
            FileTreeEntry::File { path, .. } => path,
        }
    }

    fn name(&self) -> &str {
        match self {
            FileTreeEntry::Directory { name, .. } => name,
            FileTreeEntry::File { name, .. } => name,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

impl State {
    fn new() -> Self {
        let home_dir = if let Some(dir) = dirs::home_dir() {
            dir
        } else {
            PathBuf::from("/")
        };
        let rencents = if let Some(dir) = dirs::data_dir() {
            if dir.join("recent.json").exists() {
                RecentManager::load_from_file(dir.join("recent.json"))
                    .unwrap_or(RecentManager::new(20))
            } else {
                RecentManager::new(20)
            }
        } else {
            RecentManager::new(20)
        };
        let recent_items: Vec<RecentItem> = Vec::from(rencents.get_recent_items());
        let mut state = State {
            current_path: home_dir.clone(),
            current_image: None,
            root_file_tree_entry: vec![
                FileTreeEntry::Directory {
                    name: "Recents".to_string(),
                    path: PathBuf::from("__RECENTS__"),
                    children: recent_items
                        .clone()
                        .iter()
                        .map(|item| FileTreeEntry::File {
                            name: item.name(),
                            path: item.path().clone(),
                        })
                        .collect(),
                    expanded: false,
                    children_loaded: false,
                },
                FileTreeEntry::Directory {
                    name: home_dir
                        .clone()
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .into_owned(),
                    path: home_dir.clone(),
                    children: vec![],
                    expanded: false,
                    children_loaded: false, // 初始状态未加载子节点
                },
            ],
            image_collection: Vec::new(), // 初始化图片库为空
            current_image_index: 0,       // 初始图片索引为 0
            resampling_bar_opened: false,
            slider_value: 50,                                  // 初始缩放条值为 50
            resampling_type: ResamplingType::Lanczos3,         // 默认使用Lanczos3算法
            original: None,                                    // 用于存储原始图片
            scaled_bytes: Vec::new(),                          // 用于存储缩放后的图片字节
            thumbnail_cache: std::collections::HashMap::new(), // 初始化缩略图缓存
            is_dragging: false,                                // 初始状态不是拖动
            last_resize_time: std::time::Instant::now(),       // 初始化时间
            preview_scaled_bytes: Vec::new(),                  // 初始化预览缩放字节
            final_scaled_bytes: Vec::new(),                    // 初始化最终缩放字节
            is_resampling_mode: false,                         // 初始状态不是缩放模式
            hand_tool_active: false,                           // 初始状态未启用手型工具
            is_panning: false,                                 // 初始状态未拖动画布
            pan_start_position: None,                          // 初始拖动开始位置
            pan_offset: iced::Vector::new(0.0, 0.0),           // 初始拖动偏移量
            recent_manager: rencents,
            is_fullscreen: false, // 初始状态不是全屏模式
        };
        load_directory_children(&mut state.root_file_tree_entry[1], home_dir.clone());
        state
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectImage => {
                let path = FileDialog::new()
                    .add_filter("image", &["png", "jpg", "jpeg", "gif", "svg"])
                    .set_directory("/")
                    .pick_file();
                self.current_image = path;
                self.current_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
                self.image_collection.clear(); // 清空图片库
                self.current_image_index = 0; // 重置图片索引
                let path = self.current_image.clone();
                Task::perform(
                    async move { Message::LoadImage(path.unwrap_or_default()) },
                    |msg| msg,
                )
            }
            Message::ChangeDirectory(path) => {
                if path.is_dir() {
                    self.current_path = path;
                } else {
                    eprintln!("Error: {} is not a directory", path.display());
                }
                Task::none()
            }
            Message::NoOp => Task::none(),
            Message::ExpandDirectory(path) => {
                // 检查是否是 Recents 目录
                let is_recents = path == PathBuf::from("__RECENTS__");

                // 现在目录树根节点有recent和home目录两个，需要遍历查找
                let needs_load = {
                    let mut found = false;
                    // 遍历所有根节点（包括recent和home）
                    for root in self.root_file_tree_entry.iter_mut() {
                        if let Some(entry) = find_entry_by_path(root, &path) {
                            match entry {
                                FileTreeEntry::Directory {
                                    expanded,
                                    children_loaded,
                                    children,
                                    ..
                                } => {
                                    *expanded = !*expanded;
                                    if *expanded {
                                        if is_recents {
                                            // Recents 目录的子项在初始化时已加载，标记为已加载
                                            *children_loaded = true;
                                            found = false;
                                        } else {
                                            // 展开且未加载 → 需要加载
                                            found = !*children_loaded;
                                        }
                                    } else {
                                        if !is_recents {
                                            // 折叠 → 清空缓存，无需加载（Recents目录保持子项）
                                            children.clear();
                                            *children_loaded = false;
                                        }
                                        found = false;
                                    }
                                }
                                _ => {}
                            }
                            break;
                        }
                    }
                    found
                };

                // 2. 需要加载时再重新借一次，只把目标节点可变引用传进去
                if needs_load {
                    for root_entry in self.root_file_tree_entry.iter_mut() {
                        if find_entry_by_path(root_entry, &path).is_some() {
                            load_directory_children(root_entry, path.clone());
                            break;
                        }
                    }
                }

                // 列出当前目录下的图片
                if !is_recents && let Ok(images) = fs::read_dir(path.clone()) {
                    self.image_collection.clear();
                    for entry in images.flatten() {
                        let child_path = entry.path();
                        if child_path.is_file() {
                            let ext = child_path.extension().map(|ext| ext.to_str());
                            let ext = ext.and_then(|s| s).unwrap_or_default().to_lowercase();
                            if ext == "png"
                                || ext == "jpg"
                                || ext == "jpeg"
                                || ext == "gif"
                                || ext == "svg"
                            {
                                self.image_collection.push(child_path);
                            }
                        }
                    }

                    // 为每个图片异步加载缩略图
                    for path in &self.image_collection {
                        if !self.thumbnail_cache.contains_key(path) {
                            let path_clone = path.clone();
                            return Task::perform(
                                async move { Message::LoadThumbnail(path_clone) },
                                |msg| msg,
                            );
                        }
                    }
                } else if is_recents {
                    // 处理 Recents 目录 - 更新图片集合为最近浏览的图片
                    self.image_collection.clear();
                    let recent_items = self.recent_manager.get_recent_items();
                    for item in recent_items {
                        self.image_collection.push(item.path().clone());
                    }

                    // 为每个图片异步加载缩略图
                    for path in &self.image_collection {
                        if !self.thumbnail_cache.contains_key(path) {
                            let path_clone = path.clone();
                            return Task::perform(
                                async move { Message::LoadThumbnail(path_clone) },
                                |msg| msg,
                            );
                        }
                    }
                }
                Task::none() // 返回空命令
            }
            Message::PickImage(path) => {
                self.current_path = path.parent().unwrap_or(&path).to_path_buf();
                self.current_image = Some(path.clone());
                self.current_image_index = self
                    .image_collection
                    .iter()
                    .position(|p| p == &path.clone())
                    .unwrap_or(0);
                let path = self.current_image.clone();
                Task::perform(
                    async move { Message::LoadImage(path.unwrap_or_default()) },
                    |msg| msg,
                )
            }
            Message::SelectFolder(path) => Task::none(),
            Message::PickNextImage => {
                if !self.image_collection.is_empty() {
                    self.current_image_index =
                        (self.current_image_index + 1) % self.image_collection.len();
                    self.current_image =
                        Some(self.image_collection[self.current_image_index].clone());
                }
                let path = self.current_image.clone();
                Task::perform(
                    async move { Message::LoadImage(path.unwrap_or_default()) },
                    |msg| msg,
                )
            }
            Message::PickPreviousImage => {
                if !self.image_collection.is_empty() {
                    if self.current_image_index == 0 {
                        self.current_image_index = self.image_collection.len() - 1;
                    } else {
                        self.current_image_index -= 1;
                    }
                    self.current_image =
                        Some(self.image_collection[self.current_image_index].clone());
                }
                let path = self.current_image.clone();
                Task::perform(
                    async move { Message::LoadImage(path.unwrap_or_default()) },
                    |msg| msg,
                )
            }
            Message::OpenResamplingBar => {
                // 这里可以添加打开缩放条的逻辑
                self.resampling_bar_opened = !self.resampling_bar_opened; // 切换缩放条状态
                Task::none()
            }
            Message::SliderChanged(value) => {
                self.slider_value = value;
                self.is_dragging = true;
                self.is_resampling_mode = true; // 进入缩放模式
                // 节流：检查距离上次缩放的时间是否超过阈值（500ms）
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(self.last_resize_time);

                if elapsed.as_millis() < 300 {
                    // 如果时间间隔太短，不执行缩放，等待下一次滑块变化
                    return Task::none();
                }

                self.last_resize_time = now;

                // 克隆所需数据，转到后台线程
                let img = self.original.clone();

                // 在拖动过程中使用Point算法（最快的算法）进行快速预览
                Task::perform(
                    async move {
                        // 在后台线程做快速缩放
                        let scaled = scale_image_async(img, value, ResamplingType::Point);
                        Message::ImageResized(scaled, false) // false表示这是预览质量
                    },
                    |msg| msg,
                )
            }

            Message::SliderReleased => {
                // 滑块释放时，安排一个延迟任务来执行高质量渲染
                // 不立即设置is_dragging = false，让FinalizeDragging来处理

                // 创建一个延迟任务，800ms后触发FinalizeDragging
                Task::perform(
                    async {
                        // 等待800ms，确保用户真的停止了拖动
                        sleep(std::time::Duration::from_millis(300));
                        Message::FinalizeDragging
                    },
                    |msg| msg,
                )
            }

            Message::FinalizeDragging => {
                if !self.is_dragging {
                    return Task::none(); // 如果已经不在拖动状态，不执行操作
                }

                self.is_dragging = false;

                // 使用高质量算法进行最终渲染
                let img = self.original.clone();
                let value = self.slider_value;
                let scale_type = self.resampling_type;

                Task::perform(
                    async move {
                        let scaled = scale_image_async(img, value, scale_type);
                        Message::ImageResized(scaled, true) // true表示这是高质量渲染
                    },
                    |msg| msg,
                )
            }
            Message::ResamplingTypeChanged(scale_type) => {
                self.resampling_type = scale_type;

                // 如果有原始图片，立即应用新算法重新缩放
                if self.original.is_some() {
                    let img = self.original.clone();
                    let value = self.slider_value;
                    return Task::perform(
                        async move {
                            let scaled = scale_image_async(img, value, scale_type);
                            Message::ImageResized(scaled, true) // 添加true表示这是高质量渲染
                        },
                        |msg| msg,
                    );
                }
                Task::none()
            }
            Message::ImageResized(scaled_bytes, is_high_quality) => {
                if is_high_quality {
                    // 高质量渲染结果，更新最终图像
                    self.final_scaled_bytes = scaled_bytes;
                    self.scaled_bytes = self.final_scaled_bytes.clone();
                } else if self.is_dragging {
                    // 如果仍在拖动，更新预览图像
                    self.preview_scaled_bytes = scaled_bytes;
                    self.scaled_bytes = self.preview_scaled_bytes.clone();
                }
                Task::none()
            }
            Message::LoadScaledBytes => {
                if !self.scaled_bytes.is_empty() {
                    let scaled = scale_image_async(
                        self.original.clone(),
                        self.slider_value,
                        self.resampling_type,
                    );
                    self.scaled_bytes = scaled;
                }
                Task::none()
            }
            Message::LoadImage(path) => {
                // Recent Image
                self.recent_manager.add_item(path.clone());
                let _ = self.recent_manager
                    .save_to_file(dirs::data_dir().unwrap().join("recent.json"));
                // 1. 加载图片
                self.is_dragging = false; // 重置拖动状态
                self.slider_value = 50; // 重置缩放条值
                self.is_resampling_mode = false; // 重置缩放模式
                self.preview_scaled_bytes.clear(); // 清空预览缓存
                self.final_scaled_bytes.clear(); // 清空最终缓存
                self.pan_offset = iced::Vector::new(0.0, 0.0); // 重置拖动偏移量
                self.is_panning = false; // 重置拖动状态
                self.pan_start_position = None; // 重置拖动开始位置

                if let Ok(img) = ImageReader::open(path.clone().as_path()).expect("open image failed").decode() {
                    let rgb_img = img.to_rgb8();
                    self.original = Some(rgb_img.clone());

                    let _ = Task::perform(async move { Message::LoadScaledBytes }, |msg| msg);
                    Task::perform(async move { Message::UploadToCloud(path.clone())}, |msg| msg)
                } else {
                    eprintln!("Failed to load image: {}", path.display());
                    Task::none()
                }
            }
            Message::LoadThumbnail(path) => {
                // 异步加载缩略图
                let path_clone = path.clone();
                Task::perform(
                    async move {
                        // 检查文件是否存在且可读
                        if !path_clone.exists() || !path_clone.is_file() {
                            eprintln!(
                                "File does not exist or is not a file: {}",
                                path_clone.display()
                            );
                            // 返回默认占位符
                            let placeholder =
                                Handle::from_rgba(80, 80, vec![200].repeat(80 * 80 * 4));
                            return Message::ThumbnailLoaded(path_clone, placeholder);
                        }

                        // 检查文件扩展名
                        let ext = path_clone
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        if !["png", "jpg", "jpeg", "gif", "bmp", "tiff", "webp"]
                            .contains(&ext.as_str())
                        {
                            eprintln!("Unsupported image format: {}", path_clone.display());
                            let placeholder =
                                Handle::from_rgba(80, 80, vec![150].repeat(80 * 80 * 4));
                            return Message::ThumbnailLoaded(path_clone, placeholder);
                        }

                        // 尝试加载图片
                        match ImageReader::open(path_clone.clone().as_path()).expect("open image failed").decode() {
                            Ok(img) => {
                                // 缩放到缩略图尺寸
                                let thumbnail =
                                    img.resize(80, 80, image::imageops::FilterType::Lanczos3);
                                let rgba = thumbnail.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                let handle = Handle::from_rgba(width, height, rgba.into_raw());
                                Message::ThumbnailLoaded(path_clone, handle)
                            }
                            Err(e) => {
                                eprintln!(
                                    "Failed to load thumbnail for {}: {}",
                                    path_clone.display(),
                                    e
                                );
                                // 返回错误占位符
                                let error_placeholder = Handle::from_rgba(
                                    80,
                                    80,
                                    vec![255, 100, 100, 255].repeat(80 * 80),
                                );
                                Message::ThumbnailLoaded(path_clone, error_placeholder)
                            }
                        }
                    },
                    |msg| msg,
                )
            }
            Message::ThumbnailLoaded(path, handle) => {
                // 缩略图加载完成，保存到缓存
                self.thumbnail_cache.insert(path.clone(), handle);

                // 检查是否还有其他缩略图需要加载
                for path in &self.image_collection {
                    if !self.thumbnail_cache.contains_key(path) {
                        let path_clone = path.clone();
                        return Task::perform(
                            async move { Message::LoadThumbnail(path_clone) },
                            |msg| msg,
                        );
                    }
                }

                Task::none()
            }
            Message::ToggleHandTool => {
                self.hand_tool_active = !self.hand_tool_active;
                Task::none()
            }
            Message::MousePressed(event) => {
                if self.hand_tool_active {
                    if let iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) = event {
                        self.is_panning = true;
                        self.pan_start_position = None; // 重置开始位置，等待第一个MouseMoved事件
                    }
                }
                Task::none()
            }
            Message::MouseReleased(event) => {
                if self.hand_tool_active {
                    if let iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left) = event {
                        self.is_panning = false;
                        self.pan_start_position = None;

                        // 拖动结束后，使用高质量算法重新渲染
                        if let Some(ref ori) = self.original {
                            let scale = self.slider_value as f32 / 50.0;
                            let final_image =
                                crop_and_scale(ori, scale, self.pan_offset, self.resampling_type);
                            self.scaled_bytes = final_image.clone();
                            self.final_scaled_bytes = final_image;
                        }
                    }
                }
                Task::none()
            }
            Message::MouseMoved(position) => {
                if self.hand_tool_active && self.is_panning {
                    if let Some(last) = self.pan_start_position {
                        let delta = Vector::new(position.x - last.x, position.y - last.y);
                        self.pan_offset = self.pan_offset + delta;
                        self.pan_start_position = Some(position);

                        // 重新裁剪+缩放
                        if let Some(ref ori) = self.original {
                            let scale = self.slider_value as f32 / 50.0;
                            let preview = crop_and_scale(
                                ori,
                                scale,
                                self.pan_offset,
                                ResamplingType::Point, // 拖动时用最快的算法
                            );
                            self.scaled_bytes = preview.clone();
                            self.preview_scaled_bytes = preview.clone();
                        }
                    } else {
                        self.pan_start_position = Some(position);
                    }
                }
                Task::none()
            }
            Message::ToggleFullscreen => {
                self.is_fullscreen = !self.is_fullscreen;
                Task::none()
            }
            Message::EscPressed => {
                if self.is_fullscreen {
                    self.is_fullscreen = false;
                }
                Task::none()
            }
            Message::UploadToCloud(path) => {
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let top_bar = container(
            row![
                // Left: App logo and title
                row![
                    text("📷").size(24).shaping(text::Shaping::Advanced),
                    text("Image Browser")
                        .size(18)
                        .color(Color::from_rgb8(33, 37, 41))
                        .font(iced::Font::MONOSPACE)
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .width(Length::Fill),
                // Center: Navigation buttons
                row![
                    button(text("Open").size(14))
                        .on_press(Message::SelectImage)
                        .style(|theme, status| button_style::default(theme, status))
                        .padding([6, 12]),
                    button(text("Next").size(14))
                        .on_press(Message::PickNextImage)
                        .style(|theme, status| button_style::default(theme, status))
                        .padding([6, 12]),
                    button(text("Previous").size(14))
                        .on_press(Message::PickPreviousImage)
                        .style(|theme, status| button_style::default(theme, status))
                        .padding([6, 12]),
                    button(text("Zoom").size(14))
                        .on_press(Message::OpenResamplingBar)
                        .style(|theme, status| button_style::default(theme, status))
                        .padding([6, 12]),
                    button(text("🖐").shaping(text::Shaping::Advanced).size(14))
                        .on_press(Message::ToggleHandTool)
                        .style(move |theme, status| {
                            if self.hand_tool_active {
                                button_style::primary(theme, status)
                            } else {
                                button_style::default(theme, status)
                            }
                        })
                        .padding([6, 12]),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center)
                .width(Length::Shrink),
                // Right: Fullscreen button
                container(
                    button(text("Fullscreen").size(14))
                        .on_press(Message::ToggleFullscreen)
                        .style(|theme, status| button_style::primary(theme, status))
                        .padding([8, 16])
                )
                .align_x(Horizontal::Right)
                .width(Length::Fill),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(20),
        )
        .padding([12, 20])
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::WHITE)),
            border: iced::Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            shadow: iced::Shadow {
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0.into(),
                color: Color::from_rgba8(0, 0, 0, 0.1),
            },
            ..Default::default()
        });
        let recent_content = self.view_file_tree(&self.root_file_tree_entry[0], 0);
        let file_tree_content = self.view_file_tree(&self.root_file_tree_entry[1], 0);

        let file_tree = container(
            scrollable(
                column![recent_content, file_tree_content]
                    .spacing(8)
                    .width(Length::Fill)
                    .padding([8, 12]),
            )
            .style(|theme, _| iced::widget::scrollable::Style {
                container: container::Style {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    ..Default::default()
                },
                vertical_rail: iced::widget::scrollable::Rail {
                    background: Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.1))),
                    border: iced::Border {
                        radius: 2.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    scroller: iced::widget::scrollable::Scroller {
                        color: Color::from_rgba8(0, 0, 0, 0.3),
                        border: iced::Border {
                            radius: 2.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                    },
                },
                horizontal_rail: iced::widget::scrollable::Rail {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    border: iced::Border::default(),
                    scroller: iced::widget::scrollable::Scroller {
                        color: Color::TRANSPARENT,
                        border: iced::Border::default(),
                    },
                },
                gap: Some(Background::Color(Color::TRANSPARENT)),
            }),
        )
        .width(280)
        .height(Length::Fill)
        .padding([16, 0])
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(Color::from_rgb8(248, 249, 250))),
            border: iced::Border {
                radius: 0.0.into(),
                width: 1.0,
                color: Color::from_rgb8(222, 226, 230),
            },
            ..Default::default()
        });

        let main_image_display: iced::Element<_> = {
            let handle = if (self.is_resampling_mode || self.hand_tool_active)
                && !self.scaled_bytes.is_empty()
            {
                // 使用当前的缩放图像（可能是预览质量或高质量）
                if self.is_dragging || self.is_panning {
                    // 如果正在拖动滑块或拖动图片，使用预览缩放后的图片
                    iced::widget::image::Handle::from_bytes(self.preview_scaled_bytes.clone())
                } else {
                    // 如果不是拖动状态，使用最终高质量缩放后的图片
                    iced::widget::image::Handle::from_bytes(self.final_scaled_bytes.clone())
                }
            } else {
                // 如果没有缩放后的图片，使用原始图片
                if let Some(path) = &self.current_image {
                    iced::widget::image::Handle::from_path(path)
                } else {
                    // 创建一个空白占位符
                    iced::widget::image::Handle::from_rgba(
                        400,
                        300,
                        vec![248, 249, 250, 255].repeat(400 * 300),
                    )
                }
            };

            let img: iced::widget::Image<Handle> = iced::widget::image(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(iced::ContentFit::Contain);

            // 用现代化的容器包装图片
            let positioned: Element<_> = if self.is_fullscreen {
                // 全屏模式：去除所有装饰，让图片占满屏幕
                container(img)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(0)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(Color::BLACK)),
                        border: iced::Border::default(),
                        shadow: iced::Shadow::default(),
                        ..Default::default()
                    })
                    .into()
            } else {
                // 非全屏模式：保持原有装饰样式
                container(img)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(20)
                    .style(|_theme| container::Style {
                        background: Some(Background::Color(Color::from_rgb8(250, 250, 250))),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: Color::from_rgb8(222, 226, 230),
                        },
                        shadow: iced::Shadow {
                            offset: Vector::new(0.0, 2.0),
                            blur_radius: 8.0.into(),
                            color: Color::from_rgba8(0, 0, 0, 0.1),
                        },
                        ..Default::default()
                    })
                    .into()
            };

            // 只在打开时显示滑块和算法选择
            let slider_layer = if self.resampling_bar_opened {
                // 手动添加每个按钮，避免使用extend和map
                let buttons = ResamplingType::all()
                    .iter()
                    .map(|&resampling_type| {
                        let is_selected = resampling_type == self.resampling_type;
                        let btn: Element<_> = button(text(resampling_type.name()).size(12))
                            .padding([6, 12])
                            .style(move |theme, status| {
                                if is_selected {
                                    button_style::primary(theme, status)
                                } else {
                                    button_style::default(theme, status)
                                }
                            })
                            .on_press(Message::ResamplingTypeChanged(resampling_type))
                            .into();
                        btn
                    })
                    .collect::<Vec<_>>();

                let algorithm_buttons = row(buttons).spacing(8).padding([8, 0]);

                // 组合滑块和算法选择
                container(
                    column![
                        text("Zoom Level")
                            .size(14)
                            .color(Color::from_rgb8(52, 58, 64)),
                        slider(50..=150, self.slider_value, Message::SliderChanged)
                            .default(50)
                            .shift_step(5)
                            .on_release(Message::SliderReleased)
                            .style(|theme, _| iced::widget::slider::Style {
                                rail: iced::widget::slider::Rail {
                                    backgrounds: (
                                        Background::Color(Color::from_rgb8(222, 226, 230)),
                                        Background::Color(Color::from_rgb8(13, 110, 253)),
                                    ),
                                    width: 4.0,
                                    border: iced::Border {
                                        color: Color::TRANSPARENT,
                                        width: 2.0.into(),
                                        radius: 2.0.into()
                                    },
                                },
                                handle: iced::widget::slider::Handle {
                                    shape: iced::widget::slider::HandleShape::Circle {
                                        radius: 8.0.into()
                                    },
                                    background: Background::Color(Color::WHITE),
                                    border_color: Color::from_rgb8(13, 110, 253),
                                    border_width: 2.0,
                                },
                            }),
                        text("Resampling Algorithm")
                            .size(14)
                            .color(Color::from_rgb8(52, 58, 64)),
                        algorithm_buttons
                    ]
                    .spacing(12),
                )
                .width(380)
                .padding(16)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::WHITE)),
                    border: iced::Border {
                        radius: 12.0.into(),
                        width: 1.0,
                        color: Color::from_rgb8(222, 226, 230),
                    },
                    shadow: iced::Shadow {
                        offset: Vector::new(0.0, 4.0),
                        blur_radius: 12.0.into(),
                        color: Color::from_rgba8(0, 0, 0, 0.15),
                    },
                    ..Default::default()
                })
                .center_x(380)
                .align_y(iced::alignment::Vertical::Top)
                .into()
            } else {
                iced::Element::new(iced::widget::Space::new(0, 0))
            };

            // 如果启用了手型工具，包装图片在MouseArea中以捕获鼠标事件
            let image_with_mouse_events: Element<_> = if self.hand_tool_active {
                iced::widget::mouse_area(positioned)
                    .on_press(Message::MousePressed(iced::mouse::Event::ButtonPressed(
                        iced::mouse::Button::Left,
                    )))
                    .on_release(Message::MouseReleased(iced::mouse::Event::ButtonReleased(
                        iced::mouse::Button::Left,
                    )))
                    .on_move(Message::MouseMoved)
                    .into()
            } else {
                positioned
            };

            if self.is_fullscreen {
                // 全屏模式：简化布局，只显示图片和必要的滑块
                container(
                    Stack::new()
                        .push(image_with_mouse_events) // 底层：带鼠标事件的图片
                        .push(slider_layer), // 顶层：滑块
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(0)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::BLACK)),
                    ..Default::default()
                })
                .into()
            } else {
                // 非全屏模式：保持原有样式
                container(
                    Stack::new()
                        .push(image_with_mouse_events) // 底层：带鼠标事件的图片
                        .push(slider_layer), // 顶层：滑块
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(16)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(Color::from_rgb8(248, 249, 250))),
                    ..Default::default()
                })
                .into()
            }
        };

        let images = self.image_collection.clone();

        // 创建缩略图标题栏
        let thumbnail_header = container(
            row![
                text("Tolores")
                    .size(16)
                    .color(Color::from_rgb8(33, 37, 41))
                    .font(iced::Font::MONOSPACE),
                container(text("")).width(Length::Fill),
                button(text("⚙").shaping(text::Shaping::Advanced))
                    .style(|theme, status| button_style::transparent(theme, status))
                    .padding([4, 8])
                    .on_press(Message::NoOp),
                button(text("🗐").shaping(text::Shaping::Advanced))
                    .style(|theme, status| button_style::transparent(theme, status))
                    .padding([4, 8])
                    .on_press(Message::NoOp),
                button(text("✕").shaping(text::Shaping::Advanced))
                    .style(|theme, status| button_style::transparent(theme, status))
                    .padding([4, 8])
                    .on_press(Message::NoOp),
            ]
            .align_y(iced::Alignment::Center)
            .spacing(8),
        )
        .padding([12, 16])
        .style(|_theme| container::Style {
            background: Some(Background::Color(Color::WHITE)),
            border: iced::Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        });

        // 生成缩略图行
        let thumbnails_row = row![]
            .spacing(12)
            .padding([0, 16])
            .width(Length::Shrink)
            .extend(images.into_iter().enumerate().map(|(idx, p)| {
                let is_selected = idx == self.current_image_index;

                let image_handle = if let Some(handle) = self.thumbnail_cache.get(&p) {
                    handle.clone()
                } else {
                    Handle::from_rgba(80, 80, vec![248, 249, 250, 255].repeat(80 * 80))
                };

                button(
                    iced::widget::image(image_handle)
                        .width(Length::Fixed(80.0))
                        .height(Length::Fixed(80.0))
                        .content_fit(iced::ContentFit::Cover),
                )
                .style(move |theme, status| {
                    if is_selected {
                        button_style::thumbnail_selected(theme, status)
                    } else {
                        button_style::thumbnail(theme, status)
                    }
                })
                .on_press(Message::PickImage(p))
                .into()
            }));

        // 将行包装在水平滚动容器中
        let thumbnails_scroll = scrollable(thumbnails_row)
            .direction(Direction::Horizontal(scrollable::Scrollbar::new()))
            .style(|theme, _| iced::widget::scrollable::Style {
                container: container::Style {
                    background: Some(Background::Color(Color::WHITE)),
                    ..Default::default()
                },
                vertical_rail: iced::widget::scrollable::Rail {
                    background: Some(Background::Color(Color::TRANSPARENT)),
                    border: iced::Border::default(),
                    scroller: iced::widget::scrollable::Scroller {
                        color: Color::TRANSPARENT,
                        border: iced::Border::default(),
                    },
                },
                horizontal_rail: iced::widget::scrollable::Rail {
                    background: Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.05))),
                    border: iced::Border {
                        radius: 3.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    scroller: iced::widget::scrollable::Scroller {
                        color: Color::from_rgba8(0, 0, 0, 0.3),
                        border: iced::Border {
                            radius: 3.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                    },
                },
                gap: Some(Background::Color(Color::TRANSPARENT)),
            });

        let collection_display = column![thumbnail_header, thumbnails_scroll];

        if self.is_fullscreen {
            // 全屏模式：只显示图片，隐藏其他UI元素
            main_image_display
        } else {
            // 非全屏模式：显示完整界面
            let main_content = row![
                file_tree,
                column![
                    main_image_display,
                    container(collection_display)
                        .height(Length::Fixed(140.0))
                        .width(Length::Fill)
                        .style(|_theme| container::Style {
                            background: Some(Background::Color(Color::WHITE)),
                            border: iced::Border {
                                radius: 0.0.into(),
                                width: 1.0,
                                color: Color::from_rgb8(222, 226, 230),
                            },
                            shadow: iced::Shadow {
                                offset: Vector::new(0.0, -1.0),
                                blur_radius: 3.0.into(),
                                color: Color::from_rgba8(0, 0, 0, 0.05),
                            },
                            ..Default::default()
                        }),
                ]
                .width(Length::FillPortion(4)) // This column takes the remaining space
                .height(Length::Fill) // Fill remaining height
            ]
            .width(Length::Fill)
            .height(Length::Fill);

            column![top_bar, main_content,]
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::keyboard::on_key_press(|key, _modifiers| match key.as_ref() {
            keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::EscPressed),
            _ => None,
        })
    }

    fn view_file_tree(&self, entry: &FileTreeEntry, level: usize) -> Element<'_, Message> {
        let indent = (level as f32) * 16.0;

        let (icon, name, on_press_msg, is_expanded) = match entry {
            FileTreeEntry::Directory {
                path,
                name,
                expanded,
                ..
            } => {
                let folder_icon = if *expanded { "📂" } else { "📁" };
                (
                    folder_icon,
                    name.clone(),
                    Message::ExpandDirectory(path.clone()),
                    *expanded,
                )
            }
            FileTreeEntry::File { path, name } => {
                ("🖼", name.clone(), Message::PickImage(path.clone()), false)
            }
        };

        let item_button = container(
            button(
                row![
                    text(icon).shaping(text::Shaping::Advanced).size(14),
                    text(name).size(13).color(Color::from_rgb8(52, 58, 64))
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .on_press(on_press_msg)
            .width(Length::Fill)
            .style(|theme, status| button_style::sidebar_item(theme, status))
            .padding([6, 8]),
        )
        .padding(iced::Padding {
            top: 1.0,
            right: 0.0,
            bottom: 1.0,
            left: indent,
        });

        let mut item_column = column![item_button];

        if let FileTreeEntry::Directory {
            expanded, children, ..
        } = entry
        {
            if *expanded {
                for child_entry in children.iter() {
                    item_column = item_column.push(self.view_file_tree(child_entry, level + 1));
                }
            }
        }

        item_column.into()
    }
}

fn scale_image_async(
    ori_img: Option<image::RgbImage>,
    slider_value: u8,
    resampling_type: ResamplingType,
) -> Vec<u8> {
    if let Some(img) = &ori_img {
        let (w0, h0) = img.dimensions();
        // 缩放倍率：1.0 = 原始大小，2.0 = 放大两倍
        let scale = slider_value as f32 / 50.0;

        // 显示区域大小（你可以替换为实际显示区域大小）
        let display_width = w0 as u32;
        let display_height = h0 as u32;

        // 计算裁剪区域大小（原始图像中的区域）
        let crop_width = (display_width as f32 / scale) as u32;
        let crop_height = (display_height as f32 / scale) as u32;

        let crop_x = (w0.saturating_sub(crop_width)) / 2;
        let crop_y = (h0.saturating_sub(crop_height)) / 2;

        // 裁剪图像
        let cropped = img.view(crop_x, crop_y, crop_width, crop_height).to_image();

        // 将裁剪区域放大到显示区域大小
        let mut dst = vec![0; (display_width * display_height * 3) as usize];
        let mut resizer = resize::new(
            cropped.width() as usize,
            cropped.height() as usize,
            display_width as usize,
            display_height as usize,
            resize::Pixel::RGB8,
            resampling_type.to_resize_type(),
        )
        .unwrap();

        let _ = resizer.resize(cropped.as_raw().as_rgb(), dst.as_rgb_mut());

        let scaled =
            ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(display_width, display_height, dst).unwrap();

        let mut buf = Vec::new();
        scaled
            .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
            .unwrap();
        return buf;
    }
    Vec::new()
}

/// 根据当前缩放倍数 + 平移偏移量，从原图裁一块并放大到显示尺寸
fn crop_and_scale(
    ori: &image::RgbImage,
    scale: f32,     // slider_value / 50.0
    offset: Vector, // 用户拖动的像素偏移（相对于显示窗口）
    resample: ResamplingType,
) -> Vec<u8> {
    let (full_w, full_h) = ori.dimensions();

    // 1. 计算"窗口"在放大后图片上的逻辑大小
    let view_w = (full_w as f32 / scale).max(1.0); // 逻辑宽
    let view_h = (full_h as f32 / scale).max(1.0); // 逻辑高

    // 2. 计算裁剪起点（左上角）
    let center_x = full_w as f32 / 2.0;
    let center_y = full_h as f32 / 2.0;

    // 修正偏移量计算：拖动方向与裁剪方向相反，且需要根据缩放比例调整
    let crop_x = (center_x - view_w / 2.0 - offset.x / scale)
        .max(0.0)
        .min(full_w as f32 - view_w) as u32;
    let crop_y = (center_y - view_h / 2.0 - offset.y / scale)
        .max(0.0)
        .min(full_h as f32 - view_h) as u32;

    let crop_w = view_w.min(full_w as f32 - crop_x as f32) as u32;
    let crop_h = view_h.min(full_h as f32 - crop_y as f32) as u32;

    // 3. 裁剪
    let cropped = ori.view(crop_x, crop_y, crop_w, crop_h).to_image();

    // 4. 放大回显示尺寸
    let mut dst = vec![0; (full_w * full_h * 3) as usize];
    let mut resizer = resize::new(
        crop_w as usize,
        crop_h as usize,
        full_w as usize,
        full_h as usize,
        resize::Pixel::RGB8,
        resample.to_resize_type(),
    )
    .unwrap();
    let _ = resizer.resize(cropped.as_raw().as_rgb(), dst.as_rgb_mut());

    let out = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(full_w, full_h, dst).unwrap();
    let mut buf = Vec::new();
    out.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();
    buf
}

fn find_entry_by_path<'a>(
    entry: &'a mut FileTreeEntry,
    path: &PathBuf,
) -> Option<&'a mut FileTreeEntry> {
    if entry.path() == path {
        return Some(entry);
    }

    if let FileTreeEntry::Directory { children, .. } = entry {
        for child in children.iter_mut() {
            if let Some(found) = find_entry_by_path(child, path) {
                return Some(found);
            }
        }
    }

    None
}

fn load_directory_children(root_entry: &mut FileTreeEntry, target_path: PathBuf) {
    if let Some(target_entry) = find_entry_by_path(root_entry, &target_path) {
        if let FileTreeEntry::Directory { children, .. } = target_entry {
            children.clear();

            if let Ok(entries) = fs::read_dir(&target_path) {
                for entry in entries.flatten() {
                    let child_path = entry.path();
                    let child_entry = FileTreeEntry::default(child_path);

                    let image_formats = [".png", ".jpg", ".jpeg", ".gif", ".svg"];
                    if !child_entry.is_directory() {
                        let mut image_flag = false;
                        for format in image_formats {
                            if child_entry.name().ends_with(format) {
                                image_flag = true;
                            }
                        }
                        if image_flag == false {
                            continue;
                        }
                    } else if child_entry.name().starts_with('.') {
                        continue;
                    }

                    children.push(child_entry);
                }
            }
        }
    }
}

fn main() -> iced::Result {
    iced::application("Image Browser", State::update, State::view)
        .subscription(State::subscription)
        .run()
}
