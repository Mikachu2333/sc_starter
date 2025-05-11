//! 配置管理模块
//!
//! 本模块负责：
//! - 读取和解析配置文件
//! - 维护默认配置
//! - 验证配置有效性
//! - 转换配置格式

use crate::hotkeys::match_keys;
use crate::types::{KeyStringGroups, KeyVkGroups, SettingsCollection};
use ini::Ini;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
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
        mod_keys: [ModKey::Win, ModKey::Alt, ModKey::Ctrl],
        vkey: VKey::P,
    },
    KeyVkGroups {
        // 钉图快捷键：Win+Alt+Ctrl+T
        mod_keys: [ModKey::Win, ModKey::Alt, ModKey::Ctrl],
        vkey: VKey::T,
    },
    KeyVkGroups {
        // 退出程序快捷键：Win+Ctrl+Shift+Esc
        mod_keys: [ModKey::Win, ModKey::Ctrl, ModKey::Shift],
        vkey: VKey::Escape,
    },
    KeyVkGroups {
        // 打开设置快捷键：Win+Alt+Ctrl+O
        mod_keys: [ModKey::Win, ModKey::Alt, ModKey::Ctrl],
        vkey: VKey::O,
    },
];

const TIME_BOOL: bool = false;
const AUTOSTART_BOOL: bool = false;

const DEFAULT_HOTKEYS: [(&str, &str); 4] = [
    ("screen_capture", "Ctrl+Win+Alt@P"),
    ("pin_to_screen", "Ctrl+Win+Alt@T"),
    ("exit", "Win+Ctrl+Shift@VK_ESCAPE"),
    ("open_conf", "Ctrl+Win+Alt@O"),
];
const DEFAULT_GUI: &str =
    "rect,ellipse,arrow,number,line,text,mosaic,eraser,|,undo,redo,|,pin,clipboard,save,close";

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
/// * 读取INI格式的配置文件
/// * 解析快捷键设置
/// * 解析保存路径设置
/// * 当配置无效时使用默认值
pub fn read_config(conf_path: &PathBuf) -> SettingsCollection {
    // 读取INI配置文件
    let mut conf = Ini::load_from_file(conf_path).unwrap();

    // 获取或创建快捷键配置组
    let hotkey_group = match conf.section(Some("hotkey")) {
        Some(x) => x.to_owned(),
        None => {
            // 创建默认快捷键配置
            for (key, value) in DEFAULT_HOTKEYS {
                conf.with_section(Some("hotkey")).set(key, value);
            }
            conf.write_to_file(conf_path).unwrap();
            conf.section(Some("hotkey")).unwrap().to_owned()
        }
    };

    // 将配置字符串转换为KeyStringGroups结构
    let string_groups: Vec<KeyStringGroups> = hotkey_group
        .iter()
        .map(|(_, value)| {
            let mut parts = value.split('@');
            let mod_keys = parts.next().unwrap().split('+').map(String::from).collect();
            let vkey = parts.next().map(String::from).unwrap();
            KeyStringGroups { mod_keys, vkey }
        })
        .collect();

    // 将KeyStringGroups转换为KeyVkGroups，无效配置使用默认值
    let mut result_groups: Vec<KeyVkGroups> = Vec::new();
    for (i, j) in string_groups.into_iter().enumerate() {
        let (status, result) = match_keys(&j);
        if status {
            result_groups.push(result);
        } else {
            result_groups.push(DEFAULT_SETTING[i].clone());
        }
    }

    // 获取并处理路径配置
    let path_section = conf.section(Some("path".to_owned()));
    let unchecked_path = match path_section {
        Some(section) => {
            if let Some(dir) = section.get("dir") {
                // 规范化路径字符串
                dir.replace("\\", "/")
                    .replace("//", "/")
                    .trim_matches(['\\', '/', '\n', '\r', '"', '\'', ' ', '\t'])
                    .to_string()
            } else {
                // 如果没有dir配置，添加默认值
                conf.with_section(Some("path")).set("dir", "&");
                conf.write_to_file(conf_path).unwrap();
                "&".to_owned()
            }
        }
        None => {
            // 如果没有path段，创建并设置默认值
            conf.with_section(Some("path")).set("dir", "&");
            conf.write_to_file(conf_path).unwrap();
            "&".to_owned()
        }
    };
    //println!("unchecked_Path: {}", unchecked_path);

    // 解析保存路径
    let path_result = resolve_path(&unchecked_path);

    //获取并处理自启
    let startup_bool = match conf.section(Some("sundry")) {
        Some(b) => match b.get("startup") {
            Some(x) => x == "1",
            None => {
                conf.with_section(Some("sundry")).set("startup", "0");
                conf.write_to_file(conf_path).unwrap();
                AUTOSTART_BOOL
            }
        },
        None => {
            conf.with_section(Some("sundry")).set("startup", "0");
            conf.write_to_file(conf_path).unwrap();
            AUTOSTART_BOOL
        }
    };

    //获取并处理以时间保存截图
    let time_bool = match conf.section(Some("sundry")) {
        Some(b) => match b.get("time") {
            Some(x) => x == "1",
            None => {
                conf.with_section(Some("sundry")).set("time", "0");
                conf.write_to_file(conf_path).unwrap();
                TIME_BOOL
            }
        },
        None => {
            conf.with_section(Some("sundry")).set("time", "0");
            conf.write_to_file(conf_path).unwrap();
            TIME_BOOL
        }
    };

    //处理time问题
    let time_result = time_bool && path_result.exists();

    //处理gui显示
    let gui_config = match conf.section(Some("sundry")) {
        Some(b) => match b.get("gui_config") {
            Some(x) => x.to_string(),
            None => {
                conf.with_section(Some("sundry"))
                    .set("gui_config", DEFAULT_GUI);
                conf.write_to_file(conf_path).unwrap();
                DEFAULT_GUI.to_string()
            }
        },
        None => {
            conf.with_section(Some("sundry"))
                .set("gui_config", DEFAULT_GUI);
            conf.write_to_file(conf_path).unwrap();
            DEFAULT_GUI.to_string()
        }
    };

    // 返回最终配置集合
    SettingsCollection {
        keys_collection: result_groups.try_into().unwrap_or(DEFAULT_SETTING.clone()),
        path: path_result,
        time: time_result,
        auto_start: startup_bool,
        gui_conf: format!("--tool:\"{}\"", gui_config),
    }
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
