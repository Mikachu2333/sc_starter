//! 配置管理模块
//!
//! 本模块负责：
//! - 读取和解析配置文件
//! - 维护默认配置
//! - 验证配置有效性
//! - 转换配置格式

use crate::{msgbox, types::*};
use std::{collections::HashMap, fs, path::PathBuf};
use toml::Value;
use windows_hotkeys::keys::{ModKey, VKey};

/// 默认设置，当配置文件不存在或配置无效时使用
const DEFAULT_AUTOSTART: bool = false;
const DEFAULT_COMP_SCALE: [i32; 2] = [-1, 100]; // 默认压缩级别和缩放级别

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
                let _ = msgbox::warn_msgbox(
                    "Dir you give is not a valid path, so we use empty path.",
                    "",
                );
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
    default_gui.insert("long".to_owned(), "pin,clipboard,save,close".to_owned());

    // 尝试读取TOML配置文件
    let config_content = match fs::read_to_string(conf_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            // 返回默认配置
            return SettingsCollection {
                keys_collection: default_settings.clone(),
                path: PathBuf::new(),
                sundry: Sundry::default(),
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
                sundry: Sundry::default(),
                gui: default_gui,
            };
        }
    };

    // 提取快捷键配置
    let hotkey_table = match config.get("hotkey").and_then(|v| v.as_table()) {
        Some(table) => table,
        None => {
            eprintln!("Hotkey section missing in config file");
            return SettingsCollection {
                keys_collection: default_settings.clone(),
                path: PathBuf::new(),
                sundry: Sundry::default(),
                gui: default_gui,
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
        let _ = msgbox::error_msgbox(error_message, "Configuration Error");
    }

    // 返回最终配置集合
    SettingsCollection {
        keys_collection: user_settings,
        path: get_path_from_config(&config),
        sundry: get_sundry_settings(&config),
        gui: get_gui_config(default_gui, &config),
    }
}

/// 从配置中提取路径设置
///
/// ### 参数
/// - `config`: TOML配置值
///
/// ### 返回值
/// - `PathBuf`: 解析后的保存路径
///
/// ### 功能
/// - 从配置文件path段读取dir设置
/// - 对路径字符串进行规范化处理
/// - 解析特殊路径符号
/// - 如果配置缺失则使用默认值
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

/// 获取杂项设置
///
/// ### 参数
/// - `config`: TOML配置值
///
/// ### 返回值
/// - `Sundry`: 包含自启动、压缩级别和缩放级别的配置结构
///
/// ### 功能
/// - 从配置文件中读取自启动设置
/// - 从配置文件中读取图像压缩和缩放设置
/// - 验证设置值的有效性，无效时使用默认值
fn get_sundry_settings(config: &Value) -> Sundry {
    let sundry_section = config.get("sundry").and_then(|v| v.as_table());

    // 获取并处理自启动设置
    let startup_bool = sundry_section
        .and_then(|t| t.get("startup"))
        .and_then(|v| v.as_bool())
        .unwrap_or(DEFAULT_AUTOSTART);

    // 获取并处理保存质量相关设置
    let comp = sundry_section
        .and_then(|t| t.get("comp_level"))
        .and_then(|v| v.as_integer())
        .and_then(|num| {
            if num >= -1 && num <= 10 {
                Some(num as i32)
            } else {
                Some(DEFAULT_COMP_SCALE[0])
            }
        })
        .unwrap_or(DEFAULT_COMP_SCALE[0]);
    let scale = sundry_section
        .and_then(|t| t.get("scale_ratio"))
        .and_then(|v| v.as_integer())
        .and_then(|num| {
            if num >= 1 && num <= 100 {
                Some(num as i32)
            } else {
                Some(DEFAULT_COMP_SCALE[1])
            }
        })
        .unwrap_or(DEFAULT_COMP_SCALE[1]);
    Sundry {
        auto_start: startup_bool,
        comp_level: comp,
        scale_level: scale,
    }
}

/// 获取GUI配置
///
/// ### 参数
/// - `default`: 默认GUI配置
/// - `config`: TOML配置值
///
/// ### 返回值
/// - `HashMap<String, String>`: 包含normal和long模式GUI配置的映射
///
/// ### 功能
/// - 从配置文件中读取GUI设置
/// - 为normal和long模式分别设置工具栏配置
/// - 如果配置不存在则使用默认值
fn get_gui_config(default: HashMap<String, String>, config: &Value) -> HashMap<String, String> {
    let mut temp: HashMap<String, String> = HashMap::new();
    let gui_config = config
        .get("gui")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("gui_config"))
        .and_then(|v| v.as_str())
        .unwrap_or(default.get("normal").unwrap())
        .to_string();

    temp.insert("normal".to_owned(), format!(r#"--tool:"{}""#, gui_config));

    let gui_long_config = config
        .get("gui")
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
/// ### 参数
/// - `renew`: 是否创建新的快捷方式
/// - `startup_dir`: Windows启动目录路径
/// - `self_path`: 当前可执行文件路径
///
/// ### 功能
/// - 删除现有的快捷方式（如果存在）
/// - 根据`renew`参数决定是否创建新的快捷方式
/// - 自动设置为开机自启动
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
