use std::{
    collections::HashMap,
    env,
    path::{Component, PathBuf, Prefix},
};
use windows_hotkeys::keys::{ModKey, VKey};

use crate::msgbox::warn_msgbox;

/// 嵌入式 ScreenCapture 程序的相关信息与程序信息
pub static RES_HASH_SHA1: &str = "5857D9E31E9B29739FA051DF537F36E8C1986528";
pub static RES_VERSION: &str = "2.3.3";
pub static PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
pub static PKG_BUILD_TIME: &str = env!("VERGEN_BUILD_TIMESTAMP");
pub static DEBUG: bool = cfg!(debug_assertions);

/// 文件存在状态结构体
/// 用于跟踪主程序所需的关键文件状态和版本信息
#[derive(Clone, Copy, Debug, Default)]
pub struct FileExist {
    /// ScreenCapture 可执行文件是否存在
    pub exe_exist: bool,
    /// ScreenCapture 可执行文件是否为最新版本
    pub exe_latest: bool,
    /// 配置文件是否存在
    pub conf_exist: bool,
}

/// 杂项设置结构体
/// 存储程序的辅助功能配置项
#[derive(Clone, Debug)]
pub struct Sundry {
    /// 是否设置程序开机自动启动
    pub auto_start: bool,
    /// 图像压缩质量等级（-1表示无损，0-10表示压缩级别）
    pub comp_level: i32,
    /// 图像缩放比例（1-100，100表示原始大小）
    pub scale_level: i32,
    /// 语言（ref: <https://unece.org/trade/cefact/unlocode-code-list-country-and-territory>）
    pub lang: String,
}
impl Sundry {
    pub fn default() -> Self {
        Sundry {
            auto_start: false,
            comp_level: -1,
            scale_level: 100,
            lang: "CN".to_uppercase()
        }
    }
}

/// 启动应用程序配置结构体
/// 存储外部应用程序的启动信息
#[derive(Clone, Debug)]
pub struct LaunchAppConfig {
    /// 应用程序可执行文件路径
    pub path: PathBuf,
    /// 启动应用程序时的命令行参数列表
    pub args: Vec<String>,
}
impl LaunchAppConfig {
    pub fn default() -> Self {
        LaunchAppConfig {
            path: PathBuf::from("C:/Windows/System32/notepad.exe"),
            args: Vec::new(),
        }
    }
}

/// 路径配置结构体
/// 存储截图保存路径和启动应用程序配置
#[derive(Clone, Debug)]
pub struct PathConfig {
    /// 截图文件保存路径
    pub save_path: PathBuf,
    /// 外部启动应用程序配置
    pub launch_app: LaunchAppConfig,
}
impl PathConfig {
    pub fn default() -> Self {
        PathConfig {
            save_path: PathBuf::new(),
            launch_app: LaunchAppConfig::default(),
        }
    }
}

/// 完整设置集合结构体
/// 存储程序的所有配置信息，包括热键、路径、杂项和GUI设置
#[derive(Clone, Debug)]
pub struct SettingsCollection {
    /// 热键配置映射表
    /// 包含以下功能的快捷键：
    /// - screen_capture: 截屏
    /// - screen_capture_long: 长截屏
    /// - pin_to_screen: 钉图到屏幕
    /// - exit: 退出程序
    /// - open_conf: 打开配置
    /// - launch_app: 启动外部应用
    pub keys_collection: KeyVkGroups,
    /// 路径相关配置，包含截图保存路径和启动应用程序配置
    pub path: PathConfig,
    /// 杂项设置，包含自动启动、图像压缩和缩放配置
    pub sundry: Sundry,
    /// GUI工具栏配置参数，包含normal和long两种模式
    pub gui: HashMap<String, String>,
}
impl SettingsCollection {
    pub fn default() -> Self {
        // 创建一个默认的快捷键配置 HashMap
        let mut default_kvs: KeyVkGroups = HashMap::new();

        // 添加截屏快捷键
        default_kvs.insert(
            "screen_capture",
            HotkeyValue {
                mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
                vkey: VKey::P,
            },
        );

        // 添加截长屏快捷键
        default_kvs.insert(
            "screen_capture_long",
            HotkeyValue {
                mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
                vkey: VKey::L,
            },
        );

        // 添加钉图快捷键
        default_kvs.insert(
            "pin_to_screen",
            HotkeyValue {
                mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
                vkey: VKey::T,
            },
        );

        // 添加退出程序快捷键
        default_kvs.insert(
            "exit",
            HotkeyValue {
                mod_keys: vec![ModKey::Win, ModKey::Ctrl, ModKey::Shift],
                vkey: VKey::Escape,
            },
        );

        // 添加打开设置快捷键
        default_kvs.insert(
            "open_conf",
            HotkeyValue {
                mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
                vkey: VKey::O,
            },
        );

        // 添加调用程序快捷键
        default_kvs.insert(
            "launch_app",
            HotkeyValue {
                mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
                vkey: VKey::A,
            },
        );

        let mut default_gui: HashMap<String, String> = HashMap::new();
        default_gui.insert(
        "normal".to_owned(),
        "rect,ellipse,arrow,number,line,text,mosaic,eraser,|,undo,redo,|,pin,clipboard,save,close"
            .to_owned(),
    );
        default_gui.insert("long".to_owned(), "pin,clipboard,save,close".to_owned());
        SettingsCollection {
            keys_collection: default_kvs,
            path: PathConfig::default(),
            sundry: Sundry::default(),
            gui: default_gui,
        }
    }

    fn launch_valid(&self) -> bool {
        self.path.launch_app.path.exists()
    }

    /// 提取热键字符串
    fn key_str(&self, key: &str) -> String {
        self.keys_collection
            .get(key)
            .map(|v| v.to_string().replace("\"", ""))
            .unwrap()
    }

    /// 格式化 Hotkeys
    fn format_hotkeys(&self) -> String {
        let launch_line = if self.launch_valid() {
            format!("\n    Launch App:       {}", self.key_str("launch_app"))
        } else {
            String::new()
        };
        format!(
            r#"Hotkeys Settings:
    Screenshot:       {}
    Long Screenshot:  {}
    Pin Image:        {}
    Exit:             {}
    Config:           {}{}"#,
            self.key_str("screen_capture"),
            self.key_str("screen_capture_long"),
            self.key_str("pin_to_screen"),
            self.key_str("exit"),
            self.key_str("open_conf"),
            launch_line,
        )
    }

    /// 格式化 Sundry 块
    fn format_sundry(&self) -> String {
        let launch_str = {
            if self.launch_valid() {
                format!(
                    "\n    Launch App Path: {}\n    Launch App Args: <{}>",
                    &self.path.launch_app.path.to_str().unwrap(),
                    {
                        let temp = self.path.launch_app.args.join(" ");
                        if temp.trim().is_empty() {
                            "None".to_string()
                        } else {
                            format!("<{}>", temp)
                        }
                    }
                )
            } else {
                String::new()
            }
        };
        format!(
            r#"Sundry:
    Save Path:       {}{}
    Auto Startup:    {}
    Comp Level:      {}
    Scale Level:     {}
    GUI:
        Normal: {}
        Long:   {}"#,
            path_display(&self.path.save_path, "Manual Select"),
            launch_str,
            self.sundry.auto_start,
            self.sundry.comp_level,
            self.sundry.scale_level,
            self.gui.get("normal").unwrap(),
            self.gui.get("long").unwrap(),
        )
    }
}
impl std::fmt::Display for SettingsCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n----------\n{}\n\n{}\n----------\n",
            self.format_hotkeys(),
            self.format_sundry()
        )
    }
}

/// 程序路径信息结构体
/// 存储程序运行时所需的关键路径信息
#[derive(Clone, Debug)]
pub struct PathInfos {
    /// 程序根目录路径
    pub dir_path: PathBuf,
    /// ScreenCapture 可执行文件路径
    pub exe_path: PathBuf,
    /// 配置文件（config.toml）路径
    pub conf_path: PathBuf,
}
impl std::fmt::Display for PathInfos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "\n************\nRoot Dir:\t{}\nProcess Path:\t{}\nConf Path:\t{}\n************\n",
            self.dir_path.display(),
            self.exe_path.display(),
            self.conf_path.display()
        )
    }
}

/// 字符串格式的按键组合结构体
/// 用于从配置文件中读取和解析热键设置
#[derive(Clone)]
pub struct KeyStringGroups {
    /// 修饰键名称列表（如"Ctrl"、"Alt"、"Shift"等字符串）
    pub mod_keys: Vec<String>,
    /// 主键名称（如"A"、"F1"、"Escape"等字符串）
    pub vkey: String,
}

/// 热键组合值结构体
/// 存储已转换为系统API格式的热键信息
#[derive(Clone, Debug)]
pub struct HotkeyValue {
    /// 修饰键枚举列表（Windows API格式）
    pub mod_keys: Vec<ModKey>,
    /// 主键枚举值（Windows API格式）
    pub vkey: VKey,
}
impl std::fmt::Display for HotkeyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mods_str: Vec<String> = self
            .mod_keys
            .iter()
            .map(|m| format!("{:?}", m.to_string()))
            .collect();

        write!(f, "{}@{:?}", mods_str.join("+"), self.vkey.to_string())
    }
}

/// Windows API 格式的热键组合映射类型
/// 键名对应功能：
/// - "screen_capture": 截屏
/// - "screen_capture_long": 长截屏  
/// - "pin_to_screen": 钉图
/// - "exit": 退出程序
/// - "open_conf": 打开配置
/// - "launch_app": 启动应用
pub type KeyVkGroups = HashMap<&'static str, HotkeyValue>;

/// 将字符串格式的快捷键配置转换为系统API可用的按键组合
///
/// ### 参数
/// - `groups`: 包含修饰键和主键字符串的结构体
///
/// ### 返回值
/// - `(bool, Vec<ModKey>, VKey)`: 转换结果元组
///   - 第一个值：转换是否成功
///   - 第二个值：转换后的修饰键枚举数组
///   - 第三个值：转换后的主键枚举值
///
/// ### 功能
/// - 解析修饰键字符串为ModKey枚举
/// - 解析主键字符串为VKey枚举
/// - 处理无效键名并设置错误状态
pub fn match_keys(groups: &KeyStringGroups) -> (bool, Vec<ModKey>, VKey) {
    let group1 = &groups.mod_keys;
    let group2 = groups.vkey.as_ref();
    let mut results_mod: Vec<ModKey> = Vec::new();
    let mut status = true;

    // 转换主键值
    let result_vk = match VKey::from_keyname(group2) {
        Ok(vk_key) => vk_key,
        Err(_) => {
            status = false;
            VKey::OemClear
        }
    };

    let match_fn_series = matches!(
        result_vk,
        VKey::F1
            | VKey::F2
            | VKey::F3
            | VKey::F4
            | VKey::F5
            | VKey::F6
            | VKey::F7
            | VKey::F8
            | VKey::F9
            | VKey::F10
            | VKey::F11
            | VKey::F12
            | VKey::F13
            | VKey::F14
            | VKey::F15
            | VKey::F16
            | VKey::F17
            | VKey::F18
            | VKey::F19
            | VKey::F20
            | VKey::F21
            | VKey::F22
            | VKey::F23
            | VKey::F24
    );

    // 转换修饰键(如Ctrl, Alt, Shift等)
    for i in group1 {
        let tmp = match ModKey::from_keyname(i) {
            Ok(mod_key) => mod_key,
            Err(_) => {
                if match_fn_series {
                    continue;
                } else {
                    status = false;
                    ModKey::NoRepeat
                }
            }
        };
        results_mod.push(tmp);
    }

    if DEBUG {
        dbg!(status, &results_mod, &result_vk);
    }
    (status, results_mod, result_vk)
}

/// 处理和规范化路径字符串
///
/// ### 参数
/// - `str_path`: 需要处理的路径字符串
///
/// ### 返回值
/// - `String`: 规范化后的路径字符串
///
/// ### 功能
/// - 统一路径分隔符为正斜杠
/// - 移除重复的分隔符
/// - 去除首尾空白字符和多余的引号、分隔符
pub fn handle_str_path(str_path: impl ToString) -> String {
    str_path
        .to_string()
        .replace("\\", "/")
        .replace("//", "/")
        .trim()
        .trim_matches(['\\', '/', '"', '\''])
        .to_string()
}

/// 解析路径字符串为PathBuf，支持特殊符号
///
/// ### 特殊路径符号
/// - `&`: 返回空路径（表示手动选择路径）
/// - `@`: 返回用户桌面目录路径
/// - `*`: 返回用户图片文件夹路径
///
/// ### 参数
/// - `path`: 待解析的路径字符串
/// - `should_dir`: 路径是否应该是目录（true）或文件（false）
///
/// ### 返回值
/// - `PathBuf`: 解析后的规范化路径
///
/// ### 功能
/// - 处理特殊符号并返回对应系统路径
/// - 验证自定义路径的存在性和类型
/// - 对无效路径显示警告并返回空路径
/// - 返回绝对路径形式
pub fn resolve_path(path: impl ToString, should_dir: bool) -> PathBuf {
    let path = path.to_string();
    match path.as_ref() {
        "&" => PathBuf::new(),
        "@" => directories::UserDirs::new()
            .unwrap()
            .desktop_dir()
            .unwrap()
            .to_path_buf(),
        "*" => directories::UserDirs::new()
            .unwrap()
            .picture_dir()
            .unwrap()
            .to_path_buf(),
        x => {
            let mut path = PathBuf::from(x.replace("/", "\\"));
            let base_dir = env::current_dir().unwrap_or_default();

            let is_absolute = {
                let mut components = path.components();
                if let Some(Component::Prefix(prefix_component)) = components.next() {
                    let has_root_dir = matches!(components.next(), Some(Component::RootDir));
                    if DEBUG {
                        dbg!(has_root_dir);
                    }
                    if !has_root_dir {
                        false
                    } else {
                        if DEBUG {
                            dbg!(prefix_component.kind());
                        }

                        matches!(
                            prefix_component.kind(),
                            Prefix::VerbatimUNC(..)
                                | Prefix::UNC(..)
                                | Prefix::VerbatimDisk(..)
                                | Prefix::Disk(_)
                                | Prefix::DeviceNS(..)
                                | Prefix::Verbatim(_)
                        )
                    }
                } else {
                    path.is_absolute()
                }
            };

            if !is_absolute {
                if path.starts_with("~") {
                    if let Ok(home_dir) = std::env::var("USERPROFILE") {
                        let mut new_path = PathBuf::from(home_dir);
                        let remaining = path.strip_prefix("~/").ok();
                        if let Some(rem) = remaining {
                            new_path.push(rem);
                            path = new_path;
                        } else if *path == *"~" {
                            path = new_path;
                        } else {
                            // "~something"
                            path = base_dir.join(path);
                        }
                    }
                } else if path.starts_with(".") {
                    let remaining = path.strip_prefix(".\\").ok();
                    path = base_dir.join(remaining.unwrap());
                } else {
                    path = base_dir.join(path);
                }
            }

            let canonical = path.canonicalize().unwrap_or_default();
            if canonical.exists() {
                canonical
            } else {
                if should_dir {
                    warn_msgbox(
                        format!("{}\nPath is invalid, use EMPTY as default.", path.display()),
                        "Warn Path Invalid",
                        5,
                    );
                }
                PathBuf::new()
            }
        }
    }
}

/// 格式化路径显示字符串
///
/// ### 参数
/// - `path`: 要显示的路径
/// - `info`: 当路径为空时显示的替代信息
///
/// ### 返回值
/// - `String`: 格式化后的显示字符串
///
/// ### 功能
/// - 当路径为空时返回指定的替代信息
/// - 否则返回路径的字符串表示
fn path_display(path: &PathBuf, info: impl ToString) -> String {
    if path == &PathBuf::new() {
        info.to_string()
    } else {
        path.display().to_string()
    }
}
