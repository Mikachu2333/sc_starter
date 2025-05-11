//! 配置管理模块
//!
//! 本模块负责：
//! - 读取和解析配置文件
//! - 维护默认配置
//! - 验证配置有效性
//! - 转换配置格式

use crate::hotkeys::match_keys;
use crate::types::{KeyStringGroups, KeyVkGroups, SettingsCollection};
use std::{fs, os::windows::process::CommandExt, path::PathBuf};
use toml::Value;
use windows_hotkeys::keys::{ModKey, VKey};

/// 默认快捷键组合
/// 当配置文件不存在或配置无效时使用
///
/// 包含四组快捷键：
/// 1. 截屏：Win+Alt+Ctrl+P
/// 2. 钉图：Win+Alt+Ctrl+T
/// 3. 退出：Win+Ctrl+Shift+Esc
/// 4. 设置：Win+Alt+Ctrl+O
static DEFAULT_SETTING: [KeyVkGroups; 4] = [
    KeyVkGroups {
        // 截屏快捷键：Win+Alt+Ctrl+P
        name: "screen_capture",
        mod_keys: [ModKey::Win, ModKey::Alt, ModKey::Ctrl],
        vkey: VKey::P,
    },
    KeyVkGroups {
        // 钉图快捷键：Win+Alt+Ctrl+T
        name: "pin_to_screen",
        mod_keys: [ModKey::Win, ModKey::Alt, ModKey::Ctrl],
        vkey: VKey::T,
    },
    KeyVkGroups {
        // 退出程序快捷键：Win+Ctrl+Shift+Esc
        name: "exit",
        mod_keys: [ModKey::Win, ModKey::Ctrl, ModKey::Shift],
        vkey: VKey::Escape,
    },
    KeyVkGroups {
        // 打开设置快捷键：Win+Alt+Ctrl+O
        name: "open_conf",
        mod_keys: [ModKey::Win, ModKey::Alt, ModKey::Ctrl],
        vkey: VKey::O,
    },
];

const TIME_BOOL: bool = false;
const AUTOSTART_BOOL: bool = false;

const DEFAULT_GUI: &str =
    "rect,ellipse,arrow,number,line,text,mosaic,eraser,|,undo,redo,|,pin,clipboard,save,close";

// 辅助函数：将快捷键结构转换为配置文件字符串格式
fn key_to_string(key: &KeyVkGroups) -> String {
    let mod_keys = key
        .mod_keys
        .iter()
        .filter(|&&m| m != ModKey::NoRepeat)
        .map(|m| match m {
            ModKey::Alt => "Alt",
            ModKey::Ctrl => "Ctrl",
            ModKey::Shift => "Shift",
            ModKey::Win => "Win",
            _ => "",
        })
        .collect::<Vec<_>>()
        .join("+");

    format!("{}@{}", mod_keys, key.vkey)
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
/// - 路径分隔符统一转换为系统标准格式
fn resolve_path(path: &str) -> PathBuf {
    match path {
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
            if temp.exists() && temp.is_dir() {
                temp
            } else {
                let _ = std::process::Command::new("mshta").raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Dir you give is not a valid path, please check it.',3,'Warning',32);close()\"").spawn();
                PathBuf::new()
            }
        }
    }
}

/// 读取并解析配置文件
///
/// ### 参数
/// * `conf_path` - 配置文件路径
///
/// ### 返回值
/// * `SettingsCollection` - 包含快捷键设置和保存路径的配置集合
///
/// ### 功能
/// * 读取TOML格式的配置文件
/// * 解析快捷键设置
/// * 解析保存路径设置
/// * 当配置无效时使用默认值
pub fn read_config(conf_path: &PathBuf) -> SettingsCollection {
    // 尝试读取TOML配置文件
    let config_content = match fs::read_to_string(conf_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            // 返回默认配置
            return SettingsCollection {
                keys_collection: DEFAULT_SETTING.clone(),
                path: PathBuf::new(),
                time: false,
                auto_start: false,
                gui_conf: format!("--tool:\"{}\"", DEFAULT_GUI),
            };
        }
    };

    // 解析TOML内容
    let config: Value = match config_content.parse() {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Failed to parse config file: {}", e);
            // 返回默认配置
            return SettingsCollection {
                keys_collection: DEFAULT_SETTING.clone(),
                path: PathBuf::new(),
                time: false,
                auto_start: false,
                gui_conf: format!("--tool:\"{}\"", DEFAULT_GUI),
            };
        }
    };

    // 提取快捷键配置
    let hotkey_table = match config.get("hotkey").and_then(|v| v.as_table()) {
        Some(table) => table,
        None => {
            eprintln!("Hotkey section missing in config file");
            // 返回默认配置，但保留其他可能有效的设置
            let path = get_path_from_config(&config);
            let (time_bool, startup_bool) = get_sundry_settings(&config);
            let gui_config = get_gui_config(&config);

            return SettingsCollection {
                keys_collection: DEFAULT_SETTING.clone(),
                path,
                time: time_bool,
                auto_start: startup_bool,
                gui_conf: gui_config,
            };
        }
    };

    // 将配置字符串转换为KeyStringGroups结构
    let mut string_groups: Vec<KeyStringGroups> = Vec::new();

    // 确保配置项顺序与DEFAULT_SETTING匹配
    for default_key in DEFAULT_SETTING.iter() {
        if let Some(value) = hotkey_table.get(default_key.name).and_then(|v| v.as_str()) {
            // 提取修饰键和主键
            let parts: Vec<&str> = value.split('@').collect();
            if parts.len() != 2 {
                // 格式错误，使用默认值
                eprintln!("Invalid hotkey format for {}: {}", default_key.name, value);
                let default_value = key_to_string(default_key);
                let parts: Vec<&str> = default_value.split('@').collect();
                let mod_keys = parts[0].split('+').map(String::from).collect();
                let vkey = parts[1].to_string();
                string_groups.push(KeyStringGroups { mod_keys, vkey });
            } else {
                let mod_keys = parts[0].split('+').map(String::from).collect();
                let vkey = parts[1].to_string();
                string_groups.push(KeyStringGroups { mod_keys, vkey });
            }
        } else {
            // 如果配置中缺少该项，使用默认值
            let default_value = key_to_string(default_key);
            let parts: Vec<&str> = default_value.split('@').collect();
            let mod_keys = parts[0].split('+').map(String::from).collect();
            let vkey = parts[1].to_string();
            string_groups.push(KeyStringGroups { mod_keys, vkey });
        }
    }

    // 将KeyStringGroups转换为KeyVkGroups，无效配置使用默认值
    let mut result_groups: Vec<KeyVkGroups> = Vec::new();
    for (i, string_group) in string_groups.into_iter().enumerate() {
        let (status, mut result) = match_keys(&string_group);
        if status {
            // 设置正确的名称
            result.name = DEFAULT_SETTING[i].name;
            result_groups.push(result);
        } else {
            // 无效配置，使用默认值
            result_groups.push(DEFAULT_SETTING[i].clone());
        }
    }

    // 获取路径和其他设置
    let path_result = get_path_from_config(&config);
    let (time_bool, startup_bool) = get_sundry_settings(&config);
    let gui_config = get_gui_config(&config);

    // 返回最终配置集合
    SettingsCollection {
        keys_collection: result_groups.try_into().unwrap_or(DEFAULT_SETTING.clone()),
        path: path_result.clone(),
        time: time_bool && path_result.exists(),
        auto_start: startup_bool,
        gui_conf: gui_config,
    }
}

// 从配置中提取路径设置
fn get_path_from_config(config: &Value) -> PathBuf {
    let path_section = config.get("path").and_then(|v| v.as_table());
    let unchecked_path = match path_section {
        Some(section) => {
            if let Some(dir) = section.get("dir").and_then(|v| v.as_str()) {
                // 规范化路径字符串
                dir.replace("\\", "/")
                    .replace("//", "/")
                    .trim_matches(['\\', '/', '\n', '\r', '"', '\'', ' ', '\t'])
                    .to_string()
            } else {
                // 如果没有dir配置，使用默认值
                "&".to_owned()
            }
        }
        None => {
            // 如果没有path段，使用默认值
            "&".to_owned()
        }
    };

    // 解析保存路径
    resolve_path(&unchecked_path)
}

// 获取sundry设置
fn get_sundry_settings(config: &Value) -> (bool, bool) {
    let sundry_section = config.get("sundry").and_then(|v| v.as_table());

    // 获取并处理以时间保存截图设置
    let time_bool = sundry_section
        .and_then(|t| t.get("time"))
        .and_then(|v| v.as_bool())
        .unwrap_or(TIME_BOOL);

    // 获取并处理自启动设置
    let startup_bool = sundry_section
        .and_then(|t| t.get("startup"))
        .and_then(|v| v.as_bool())
        .unwrap_or(AUTOSTART_BOOL);

    (time_bool, startup_bool)
}

// 获取GUI配置
fn get_gui_config(config: &Value) -> String {
    let gui_config = config
        .get("sundry")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("gui_config"))
        .and_then(|v| v.as_str())
        .unwrap_or(DEFAULT_GUI)
        .to_string();

    format!("--tool:\"{}\"", gui_config)
}

/// 设置或更新启动时运行的快捷方式
///
/// 此函数旨在根据给定的参数在指定的启动目录中创建或删除程序的快捷方式
/// 它首先尝试删除任何现有的快捷方式，然后根据`renew`参数决定是否创建新的快捷方式
///
/// 参数:
/// - `renew`: 一个布尔值，指示是否创建新的快捷方式
/// - `startup_dir`: 一个引用，指向包含启动快捷方式的目录的PathBuf对象
/// - `self_path`: 一个引用，指向当前可执行文件路径的PathBuf对象
pub fn set_startup(renew: bool, startup_dir: &PathBuf, self_path: &PathBuf) {
    // 生成快捷方式的名称，基于当前可执行文件的主名称
    let lnk_name = format!("{}.lnk", self_path.file_stem().unwrap().to_str().unwrap());
    // 构建启动目录中快捷方式的完整路径
    let startup_path = startup_dir.join(lnk_name);
    //println!("{}", startup_path.display());

    // 如果快捷方式已经存在，则尝试删除它
    if startup_path.exists() {
        std::fs::remove_file(&startup_path).expect("Failed to remove old shortcut.");
        println!("Lnk Removed.");
    }

    // 如果`renew`参数为真，则尝试创建新的快捷方式
    if renew {
        mslnk::ShellLink::new(self_path)
            .unwrap()
            .create_lnk(startup_path)
            .unwrap();
        println!("Lnk Created Successfully.");
    }
}
