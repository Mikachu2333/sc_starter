use std::{collections::HashMap, path::PathBuf};
use windows_hotkeys::keys::{ModKey, VKey};

/// EMBEDDED ScreenCapture 哈希值 (SHA1)
pub const RES_HASH: &str = "9D0655C41D1C05475C458A5091D6DE01034B0C5B";
/// EMBEDDED ScreenCapture 版本号
pub const RES_VERSION: &str = "2.3.3";

/// 文件存在状态结构体
/// 用于跟踪主程序所需的关键文件状态
#[derive(Clone, Copy, Debug)]
pub struct FileExist {
    /// exe文件是否存在
    pub exe_exist: bool,
    /// exe文件是否为最新版本
    pub exe_latest: bool,
    /// 配置文件是否存在
    pub conf_exist: bool,
}
impl Default for FileExist {
    fn default() -> Self {
        FileExist {
            exe_exist: false,
            exe_latest: false,
            conf_exist: false,
        }
    }
}

/// 杂项设置结构体
/// 存储程序的各种辅助配置
#[derive(Clone, Debug)]
pub struct Sundry {
    /// 是否自动启动程序
    pub auto_start: bool,
    /// 图像压缩
    pub comp_level: i32,
    /// 图像缩放
    pub scale_level: i32,
}
impl Sundry {
    pub fn default() -> Self {
        Sundry {
            auto_start: false,
            comp_level: -1,
            scale_level: 100,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LaunchAppConfig {
    pub path: PathBuf,
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

#[derive(Clone, Debug)]
pub struct PathConfig {
    pub save_path: PathBuf,
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

/// 设置集合结构体
/// 存储程序的所有配置信息
#[derive(Clone, Debug)]
pub struct SettingsCollection {
    /// 热键配置数组
    /// 1. 截屏(screen_capture)
    /// 2. 截长屏(screen_capture_long)
    /// 3. 钉图(pin_to_screen)
    /// 4. 退出(exit)
    /// 5. 设置(open_conf)
    pub keys_collection: KeyVkGroups,
    /// 路径设置，包含保存路径以及启动app的路径
    pub path: PathConfig,
    /// 杂项设置
    /// 包含自动启动、图像压缩和缩放设置
    pub sundry: Sundry,
    /// GUI配置参数
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
}
impl std::fmt::Display for SettingsCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let get_key_str = |key: &str| -> String {
            self.keys_collection
                .get(key)
                .map(|v| v.to_string().replace("\"", ""))
                .unwrap()
        };

        write!(
            f,
            r#"
----------
Hotkeys Settings:
    Screenshot:       {}
    Long Screenshot:  {}
    Pin Image:        {}
    Exit:             {}
    Config:           {}

Sundry:
    Save Path:\t\t{}
    Launch App Path:\t{}
    Launch App Args:\t{}
    Auto Startup:\t{}
    Comp Level:\t\t{}
    Scale Level:\t\t{}
    GUI:
        Normal: {}
        Long:   {}
----------
"#,
            get_key_str("screen_capture"),
            get_key_str("screen_capture_long"),
            get_key_str("pin_to_screen"),
            get_key_str("exit"),
            get_key_str("open_conf"),
            self.path.save_path.display(),
            self.path.launch_app.path.display(),
            self.path.launch_app.args.join(" "),
            self.sundry.auto_start,
            self.sundry.comp_level,
            self.sundry.scale_level,
            self.gui.get("normal").unwrap(),
            self.gui.get("long").unwrap()
        )
    }
}

/// 路径信息结构体
/// 存储程序运行所需的所有路径
#[derive(Clone, Debug)]
pub struct PathInfos {
    /// 程序运行目录
    pub dir_path: PathBuf,
    /// 截图程序exe路径
    pub exe_path: PathBuf,
    /// 配置文件路径
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
/// 用于从配置文件中读取的原始热键设置
#[derive(Clone)]
pub struct KeyStringGroups {
    /// 修饰键列表（如Ctrl、Alt、Shift等）
    pub mod_keys: Vec<String>,
    /// 主键（如A-Z、F1-F12等）
    pub vkey: String,
}

/// 单个热键组合的值部分
#[derive(Clone, Debug)]
pub struct HotkeyValue {
    /// 修饰键
    pub mod_keys: Vec<ModKey>,
    /// 主键
    pub vkey: VKey,
}
impl HotkeyValue {
    pub fn to_string(&self) -> String {
        let mods_str: Vec<String> = self
            .mod_keys
            .iter()
            .map(|m| format!("{:?}", m.to_string()))
            .collect();

        format!("{}@{:?}", mods_str.join("+"), self.vkey.to_string())
    }
}

/// Windows API格式的按键组合映射
/// 用于实际注册系统热键
/// 键名对应：screen_capture, screen_capture_long, pin_to_screen, exit, open_conf
pub type KeyVkGroups = HashMap<&'static str, HotkeyValue>;

/// 将快捷键配置字符串转换为系统可用的按键组合
///
/// ### 参数
/// - `groups`: 包含按键字符串的结构体
///
/// ### 返回值
/// - `(bool, Vec<ModKey>, VKey)`: 转换状态和结果
/// - 第一个值表示转换是否成功
/// - 第二个值为转换后的修饰键数组
/// - 第三个值为转换后的主键值
pub fn match_keys(groups: &KeyStringGroups) -> (bool, Vec<ModKey>, VKey) {
    let group1 = &groups.mod_keys;
    let group2 = groups.vkey.as_ref();
    let mut results_mod: Vec<ModKey> = Vec::new();
    let mut status = true;

    // 转换修饰键(如Ctrl, Alt, Shift等)
    for i in group1 {
        let tmp = match ModKey::from_keyname(i) {
            Ok(mod_key) => mod_key,
            Err(_) => {
                status = false;
                ModKey::NoRepeat
            }
        };
        results_mod.push(tmp);
    }

    // 转换主键值
    let result_vk = match VKey::from_keyname(group2) {
        Ok(vk_key) => vk_key,
        Err(_) => {
            status = false;
            VKey::OemClear
        }
    };

    (status, results_mod, result_vk)
}

pub fn handle_str_path(str_path: impl ToString) -> String {
    str_path
        .to_string()
        .replace("\\", "/")
        .replace("//", "/")
        .trim()
        .trim_matches(['\\', '/', '"', '\''])
        .to_string()
}

/// 解析路径字符串为PathBuf
///
/// ### 特殊路径符号
/// - `&`: 返回空路径（手动选择）
/// - `@`: 返回桌面路径
/// - `*`: 返回图片文件夹路径
///
/// ### 参数
/// - `path`: 待解析的路径字符串
///
/// ### 返回值
/// - `PathBuf`: 解析后的路径
///
/// ### 说明
/// - 自定义路径必须存在且为目录
/// - 无效路径时显示警告弹窗并返回空路径
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
            // 验证路径是否存在
            let temp = PathBuf::from(x);

            if temp.exists() && (temp.is_dir() == should_dir) {
                temp.canonicalize().unwrap()
            } else {
                let _ = crate::msgbox::warn_msgbox(
                    "Dir you give is not a valid path, so we use empty path.",
                    "",
                );
                PathBuf::new()
            }
        }
    }
}
