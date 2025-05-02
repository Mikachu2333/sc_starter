use crate::hotkeys::match_keys;
use crate::types::{KeyStringGroups, KeyVkGroups, SettingsCollection};
use ini::Ini;
use std::path::PathBuf;
use windows_hotkeys::keys::{ModKey, VKey};

/// 默认快捷键设置
/// 当配置文件不存在或配置无效时使用
static DEFAULT_SETTING: [KeyVkGroups; 4] = [
    KeyVkGroups {
        // 截屏快捷键：Win+Alt+Ctrl+P
        mod_keys: [ModKey::Win, ModKey::Alt, ModKey::Ctrl],
        vkey: VKey::P,
    },
    KeyVkGroups {
        // 钉图快捷键：Win+Alt+Ctrl+C
        mod_keys: [ModKey::Win, ModKey::Alt, ModKey::Ctrl],
        vkey: VKey::C,
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

const DEFAULT_HOTKEYS: [(&str, &str); 4] = [
    ("screen_capture", "Ctrl+Win+Alt@P"),
    ("pin_to_screen", "Ctrl+Win+Alt@T"),
    ("exit", "Win+Ctrl+Shift@VK_ESCAPE"),
    ("open_conf", "Ctrl+Win+Alt@O"),
];

/// 解析路径字符串为PathBuf
///
/// ### 参数
/// * `path` - 待解析的路径字符串
///
/// ### 返回值
/// * 解析后的路径
///
/// ### 特殊路径符号
/// * "&" - 返回空路径
/// * "@" - 返回桌面路径
/// * "*" - 返回图片文件夹路径
/// * 其他 - 直接转换为PathBuf
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
            if temp.exists() {
                temp
            } else {
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
    let conf = Ini::load_from_file(conf_path).unwrap();

    // 获取或创建快捷键配置组
    let hotkey_group = match conf.section(Some("hotkey".to_owned())) {
        Some(x) => x.to_owned(),
        None => {
            let mut props = ini::Properties::new();
            for (key, value) in DEFAULT_HOTKEYS.iter() {
                props.insert(key.to_string(), value.to_string());
            }
            props
        }
    };

    // 读取并处理保存路径设置
    let unchecked_path = conf
        .section(Some("path"))
        .and_then(|section| section.get("dir"))
        .map(|path| {
            // 规范化路径字符串
            path.replace("\\", "/")
                .replace("//", "/")
                .trim_matches(['\\', '/', '\n', '\r', '"', '\'', ' ', '\t'])
                .to_string()
        })
        .unwrap_or("&".to_owned());
    //println!("UncheckedPath: {}", &unchecked_path);

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

    // 解析保存路径
    let path_result = resolve_path(&unchecked_path);

    // 返回最终配置集合
    SettingsCollection {
        keys_collection: result_groups.try_into().unwrap_or(DEFAULT_SETTING.clone()),
        path: path_result,
    }
}
