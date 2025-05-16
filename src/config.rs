//! 配置管理模块
//!
//! 本模块负责：
//! - 读取和解析配置文件
//! - 维护默认配置
//! - 验证配置有效性
//! - 转换配置格式

use crate::types::*;
use std::{collections::HashMap, fs, os::windows::process::CommandExt, path::PathBuf};
use toml::Value;
use windows_hotkeys::keys::{ModKey, VKey};

/// 默认快捷键组合
/// 当配置文件不存在或配置无效时使用
///
/// 包含五组快捷键：
/// 1. 截屏：Win+Alt+Ctrl+P
/// 2. 截长屏：Win+Alt+Ctrl+L
/// 3. 钉图：Win+Alt+Ctrl+T
/// 4. 退出：Win+Ctrl+Shift+Esc
/// 5. 设置：Win+Alt+Ctrl+O

const AUTOSTART_BOOL: bool = false;

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
    // 创建一个默认的快捷键配置 HashMap
    let mut default_settings: KeyVkGroups = HashMap::new();

    // 添加截屏快捷键
    default_settings.insert(
        "screen_capture",
        HotkeyValue {
            mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
            vkey: VKey::P,
        },
    );

    // 添加截长屏快捷键
    default_settings.insert(
        "screen_capture_long",
        HotkeyValue {
            mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
            vkey: VKey::L,
        },
    );

    // 添加钉图快捷键
    default_settings.insert(
        "pin_to_screen",
        HotkeyValue {
            mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
            vkey: VKey::T,
        },
    );

    // 添加退出程序快捷键
    default_settings.insert(
        "exit",
        HotkeyValue {
            mod_keys: vec![ModKey::Win, ModKey::Ctrl, ModKey::Shift],
            vkey: VKey::Escape,
        },
    );

    // 添加打开设置快捷键
    default_settings.insert(
        "open_conf",
        HotkeyValue {
            mod_keys: vec![ModKey::Win, ModKey::Alt, ModKey::Ctrl],
            vkey: VKey::O,
        },
    );

    let mut default_gui: HashMap<String, String> = HashMap::new();
    default_gui.insert(
        "normal".to_owned(),
        "rect,ellipse,arrow,number,line,text,mosaic,eraser,|,undo,redo,|,pin,clipboard,save,close"
            .to_owned(),
    );
    default_gui.insert(
        "long".to_owned(),
        "pin,clipboard,save,close".to_owned(),
    );

    // 尝试读取TOML配置文件
    let config_content = match fs::read_to_string(conf_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            // 返回默认配置
            return SettingsCollection {
                keys_collection: default_settings.clone(),
                path: PathBuf::new(),
                auto_start: false,
                gui: default_gui,
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
                keys_collection: default_settings.clone(),
                path: PathBuf::new(),
                auto_start: false,
                gui:default_gui,
            };
        }
    };

    // 提取快捷键配置
    let hotkey_table = match config.get("hotkey").and_then(|v| v.as_table()) {
        Some(table) => table,
        None => {
            eprintln!("Hotkey section missing in config file");
            // 返回默认配置，但保留其他可能有效的设置
            return SettingsCollection {
                keys_collection: default_settings.clone(),
                path: get_path_from_config(&config),
                auto_start: get_sundry_settings(&config),
                gui:default_gui,
            };
        }
    };

    // 将配置字符串转换为KeyStringGroups结构
    let mut user_settings: KeyVkGroups = HashMap::new();
    let mut errors: Vec<String> = Vec::new();

    for (default_k, default_v) in default_settings {
        if let Some(custom_hotkey) = hotkey_table.get(default_k).and_then(|v| v.as_str()) {
            // 提取修饰键和主键
            let parts: Vec<&str> = custom_hotkey.split('@').collect();
            if parts.len() != 2 {
                // 格式错误，使用默认值
                let error_message =
                    format!("Invalid hotkey format for {}: {}", default_k, custom_hotkey);
                eprintln!("{}", error_message);
                errors.push(error_message);
                user_settings.insert(default_k, default_v.clone());
            } else {
                let temp = KeyStringGroups {
                    mod_keys: parts[0].split('+').map(String::from).collect(),
                    vkey: parts[1].to_string(),
                };

                match match_keys(&temp) {
                    (true, mvks, vk) => {
                        user_settings.insert(
                            default_k,
                            HotkeyValue {
                                mod_keys: mvks,
                                vkey: vk,
                            },
                        );
                    }
                    (false, _, _) => {
                        // Invalid configuration, use default value
                        let error_message = format!(
                            "Invalid hotkey configuration for {}: {}",
                            default_k, custom_hotkey
                        );
                        eprintln!("{}", error_message);
                        errors.push(error_message);
                        user_settings.insert(default_k, default_v.clone());
                    }
                }
            }
        } else {
            // 如果配置中缺少该项，使用默认值
            user_settings.insert(default_k, default_v.clone());
        }
    }

    // 在配置处理后通知用户错误
    if !errors.is_empty() {
        let error_message = format!("配置文件中存在以下问题:\n{}", errors.join("\n"));
        let _ = std::process::Command::new("mshta")
            .raw_arg(&format!("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('{}',10,'Configuration Errors',48);close()\"", 
                error_message.replace("\"", "'").replace("\n", "\\n")))
            .spawn();
    }

    // 返回最终配置集合
    SettingsCollection {
        keys_collection: user_settings,
        path: get_path_from_config(&config),
        auto_start: get_sundry_settings(&config),
        gui: get_gui_config(default_gui,&config),
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
                    .trim()
                    .trim_matches(['\\', '/', '"', '\''])
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
fn get_sundry_settings(config: &Value) -> bool {
    let sundry_section = config.get("sundry").and_then(|v| v.as_table());

    // 获取并处理自启动设置
    let startup_bool = sundry_section
        .and_then(|t| t.get("startup"))
        .and_then(|v| v.as_bool())
        .unwrap_or(AUTOSTART_BOOL);

    startup_bool
}

// 获取GUI配置
fn get_gui_config(default:HashMap<String,String>,config: &Value) -> HashMap<String, String> {
    let mut temp: HashMap<String, String> = HashMap::new();
    let gui_config = config
        .get("sundry")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("gui_config"))
        .and_then(|v| v.as_str())
        .unwrap_or(default.get("normal").unwrap())
        .to_string();

    temp.insert("normal".to_owned(), format!(r#"--tool:"{}""#, gui_config));

    let gui_long_config = config
        .get("sundry")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("long_gui_config"))
        .and_then(|v| v.as_str())
        .unwrap_or(default.get("long").unwrap())
        .to_string();

    temp.insert(
        "long".to_owned(),
        format!(r#"--tool:"{}""#, gui_long_config),
    );
    temp
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
