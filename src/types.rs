use std::path::PathBuf;
use windows_hotkeys::keys::{ModKey, VKey};

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

/// 设置集合结构体
/// 存储程序的所有配置信息
#[derive(Clone, Debug)]
pub struct SettingsCollection {
    /// 热键配置数组，固定4组
    /// 1. 截屏
    /// 2. Pin
    /// 3. 退出
    /// 4. 设置
    pub keys_collection: [KeyVkGroups; 4],
    /// 截图保存路径
    pub path: PathBuf,
    /// 是否以截图时间保存图片
    pub time: bool,
    /// 是否自动启动程序
    pub auto_start: bool,
    /// 是否更改显示效果
    pub gui_conf: String
}
impl std::fmt::Display for SettingsCollection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n----------\nHotkeys Settings:\n  Screenshot:\t{}\n  Pin Image:\t{}\n  Exit:\t\t{}\n  Config:\t{}\n\nSave Path:\t\t{:?}\nTime-based Saving:\t{}\nAuto Startup:\t\t{}\n----------\n\n",
            self.keys_collection[0],
            self.keys_collection[1],
            self.keys_collection[2],
            self.keys_collection[3],
            self.path,
            self.time,
            self.auto_start
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

/// Windows API格式的按键组合结构体
/// 用于实际注册系统热键
#[derive(Clone, Debug)]
pub struct KeyVkGroups {
    /// 修饰键数组，固定3个
    pub mod_keys: [ModKey; 3],
    /// 主键的VK码
    pub vkey: VKey,
}
impl std::fmt::Display for KeyVkGroups {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "mod keys: {:?}  vkey: {}", self.mod_keys, self.vkey)
    }
}
