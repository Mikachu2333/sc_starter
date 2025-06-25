use std::{collections::HashMap, path::PathBuf};
use windows_hotkeys::keys::{ModKey, VKey};

/// EMBEDDED ScreenCapture Hash Value (SHA1)
pub const RES_HASH: &str = "9D0655C41D1C05475C458A5091D6DE01034B0C5B";
/// EMBEDDED ScreenCapture Version
pub const RES_VERSION: &str = "2.3.2";

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
    /// 截图保存路径
    pub path: PathBuf,
    /// 是否自动启动程序
    pub auto_start: bool,
    /// GUI配置参数
    pub gui: HashMap<String, String>,
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
    Save Path:    {}
    Auto Startup: {}
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
            self.path.display(),
            self.auto_start,
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
