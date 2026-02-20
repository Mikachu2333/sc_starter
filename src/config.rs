//! 配置管理模块
//!
//! 本模块负责：
//! - 读取和解析配置文件
//! - 维护默认配置
//! - 验证配置有效性
//! - 转换配置格式

use crate::{
    msgbox::{self, error_msgbox},
    types::*,
};
use std::{collections::HashMap, fs, path::PathBuf};
use toml::Value;

/// 读取并解析TOML配置文件
///
/// ### 参数
/// - `conf_path`: 配置文件的路径
///
/// ### 返回值
/// - `SettingsCollection`: 解析后的完整配置集合
///
/// ### 功能
/// - 读取指定路径的TOML配置文件
/// - 解析配置内容并合并默认设置
/// - 处理文件读取和解析错误
/// - 返回包含所有配置项的结构体
pub fn read_config(conf_path: &PathBuf) -> SettingsCollection {
    let default_settings = SettingsCollection::default();

    // 尝试读取TOML配置文件
    let config_content = match fs::read_to_string(conf_path) {
        Ok(content) => content
            .replace("“", "\"")
            .replace("”", "\"")
            .replace("‘", "'")
            .replace("’", "'")
            .replace("，", ",")
            .replace("。", ".")
            .replace("｜", "|")
            .replace("：", ":")
            .replace("—", "-"),
        Err(e) => {
            eprintln!("Failed to read config file: {}", e);
            // 返回默认配置
            return default_settings;
        }
    };

    // 解析TOML内容
    let config: Value = match toml::from_str(&config_content) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("Failed to parse config file: {}", e);
            error_msgbox(format!("{}", e), "Error Parse Config", 5);
            // 返回默认配置
            return default_settings;
        }
    };

    // 返回最终配置集合
    SettingsCollection {
        keys_collection: get_kvs_from_config(default_settings.keys_collection, &config),
        path: get_path_from_config(default_settings.path, &config),
        sundry: get_sundry_settings(default_settings.sundry, &config),
        gui: get_gui_config(default_settings.gui, &config),
    }
}

/// 从配置中提取快捷键设置
///
/// ### 参数
/// - `default`: 默认快捷键配置映射
/// - `config`: TOML配置值引用
///
/// ### 返回值
/// - `HashMap<&'static str, HotkeyValue>`: 解析后的快捷键配置映射
///
/// ### 功能
/// - 从配置文件hotkey段读取自定义快捷键
/// - 解析快捷键格式（修饰键@主键）
/// - 验证快捷键有效性
/// - 对无效配置使用默认值并报告错误
/// - 集中显示所有配置错误
fn get_kvs_from_config(
    default: HashMap<&'static str, HotkeyValue>,
    config: &Value,
) -> HashMap<&'static str, HotkeyValue> {
    // 将配置字符串转换为KeyStringGroups结构
    let mut user_settings: KeyVkGroups = HashMap::new();
    let mut errors: Vec<String> = Vec::new();

    // 提取快捷键配置
    let hotkey_table = match config.get("hotkey").and_then(|v| v.as_table()) {
        Some(table) => table,
        None => {
            eprintln!("Hotkey section missing in config file");
            return default;
        }
    };

    for (default_k, default_v) in default {
        if let Some(custom_hotkey) = hotkey_table.get(default_k).and_then(|v| v.as_str()) {
            // 提取修饰键和主键
            let parts: Vec<&str> = custom_hotkey.split('@').collect();
            if DEBUG {
                dbg!(&custom_hotkey);
            }
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
                if DEBUG {
                    dbg!(&temp.mod_keys, &temp.vkey);
                }

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
        msgbox::error_msgbox(error_message, "Configuration Error", 0);
    }
    user_settings
}

/// 从配置中提取路径设置
///
/// ### 参数
/// - `default`: 默认路径配置
/// - `config`: TOML配置值引用
///
/// ### 返回值
/// - `PathConfig`: 解析后的路径配置结构
///
/// ### 功能
/// - 从配置文件path段读取dir、launch_app_path和launch_app_args设置
/// - 处理路径字符串规范化和特殊符号解析
/// - 解析启动应用程序的参数（使用Tab分隔）
/// - 如果配置缺失则使用默认值
fn get_path_from_config(default: PathConfig, config: &Value) -> PathConfig {
    let path_section = config.get("path").and_then(|v| v.as_table());

    let unchecked = match path_section {
        Some(section) => {
            let str_save_path = if let Some(dir) = section.get("dir").and_then(|v| v.as_str()) {
                handle_str_path(dir)
            } else {
                default.save_path.to_string_lossy().to_string()
            };
            let str_launch_path =
                if let Some(launch) = section.get("launch_app_path").and_then(|v| v.as_str()) {
                    handle_str_path(launch)
                } else {
                    default.launch_app.path.to_string_lossy().to_string()
                };
            let str_launch_args =
                if let Some(launch) = section.get("launch_app_args").and_then(|v| v.as_str()) {
                    launch.split("\t").map(String::from).collect()
                } else {
                    default.launch_app.args
                };

            (str_save_path, str_launch_path, str_launch_args)
        }
        None => (
            default.save_path.to_string_lossy().to_string(),
            default.launch_app.path.to_string_lossy().to_string(),
            default.launch_app.args,
        ),
    };

    PathConfig {
        save_path: resolve_path(&unchecked.0, true),
        launch_app: LaunchAppConfig {
            path: resolve_path(&unchecked.1, false),
            args: unchecked.2,
        },
    }
}

/// 从配置中提取杂项设置
///
/// ### 参数
/// - `default`: 默认杂项配置
/// - `config`: TOML配置值引用
///
/// ### 返回值
/// - `Sundry`: 包含自启动、压缩级别和缩放级别的配置结构
///
/// ### 功能
/// - 从配置文件sundry段读取startup、comp_level和scale_ratio设置
/// - 验证压缩级别范围（-1到10）
/// - 验证缩放比例范围（1到100）
/// - 对超出范围的值使用默认配置
fn get_sundry_settings(default: Sundry, config: &Value) -> Sundry {
    let sundry_section = config.get("sundry").and_then(|v| v.as_table());

    // 获取并处理自启动设置
    let startup_bool = sundry_section
        .and_then(|t| t.get("startup"))
        .and_then(|v| v.as_bool())
        .unwrap_or(default.auto_start);

    // 获取并处理保存质量相关设置
    let comp = sundry_section
        .and_then(|t| t.get("comp_level"))
        .and_then(|v| v.as_integer())
        .map(|num| {
            if (-1..=10).contains(&num) {
                num as i32
            } else {
                default.comp_level
            }
        })
        .unwrap_or(default.comp_level);
    let scale = sundry_section
        .and_then(|t| t.get("scale_ratio"))
        .and_then(|v| v.as_integer())
        .map(|num| {
            if (1..=100).contains(&num) {
                num as i32
            } else {
                default.scale_level
            }
        })
        .unwrap_or(default.scale_level);
    let lang_code = sundry_section
        .and_then(|t| t.get("lang"))
        .and_then(|v| v.as_integer())
        .unwrap_or(-1);
    Sundry {
        auto_start: startup_bool,
        comp_level: comp,
        scale_level: scale,
        lang: lang_code == 1,
    }
}

/// 从配置中提取GUI设置
///
/// ### 参数
/// - `default`: 默认GUI配置映射
/// - `config`: TOML配置值引用
///
/// ### 返回值
/// - `HashMap<String, String>`: 包含normal和long模式GUI配置的映射
///
/// ### 功能
/// - 从配置文件gui段读取gui_config和long_gui_config设置
/// - 为normal和long模式分别生成工具栏参数格式
/// - 将配置值包装为命令行参数格式（--tool:"配置值"）
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

/// 管理系统启动时的快捷方式
///
/// ### 参数
/// - `renew`: 是否创建新的快捷方式（true=创建，false=仅删除）
/// - `startup_dir`: Windows启动目录路径
/// - `self_path`: 当前可执行文件路径
///
/// ### 功能
/// - 删除现有的启动快捷方式（如果存在）
/// - 根据renew参数决定是否创建新的快捷方式
/// - 快捷方式名称基于可执行文件名自动生成
/// - 用于控制程序开机自启动行为
pub fn set_startup(renew: bool, startup_dir: &std::path::Path, self_path: &PathBuf) {
    // 生成快捷方式的名称，基于当前可执行文件的主名称
    let lnk_name = match self_path.file_stem().and_then(|s| s.to_str()) {
        Some(name) => format!("{}.lnk", name),
        None => {
            eprintln!("Failed to get executable file stem for shortcut name.");
            return;
        }
    };
    // 构建启动目录中快捷方式的完整路径
    let startup_path = startup_dir.join(lnk_name);
    //println!("{}", startup_path.display());

    // 如果快捷方式已经存在，则尝试删除它
    if startup_path.exists() {
        match std::fs::remove_file(&startup_path) {
            Ok(_) => println!("Lnk Removed."),
            Err(e) => {
                eprintln!("Failed to remove old shortcut: {}", e);
                return;
            }
        }
    }

    // 如果`renew`参数为真，则尝试创建新的快捷方式
    if renew {
        match mslnk::ShellLink::new(self_path) {
            Ok(sl) => {
                if let Err(e) = sl.create_lnk(&startup_path) {
                    eprintln!("Failed to create startup shortcut: {}", e);
                } else {
                    println!("Lnk Created Successfully.");
                }
            }
            Err(e) => {
                eprintln!("Failed to create ShellLink: {}", e);
            }
        }
    }
}
