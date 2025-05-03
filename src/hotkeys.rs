//! 快捷键管理模块
//!
//! 本模块负责：
//! - 注册全局快捷键
//! - 处理快捷键事件
//! - 管理快捷键线程

use crate::file_ops::operate_exe;
use crate::types::{KeyStringGroups, KeyVkGroups, SettingsCollection};
use std::sync::mpsc;
use std::thread;
use std::{path::PathBuf, thread::JoinHandle};
use windows_hotkeys::{
    keys::{ModKey, VKey},
    singlethreaded::HotkeyManager,
    HotkeyManagerImpl,
};

/// 设置全局快捷键并返回事件发送器
pub fn set_hotkeys(
    settings_collected: &SettingsCollection,
) -> (JoinHandle<()>, mpsc::Receiver<&'static str>) {
    let settings_collected = settings_collected.clone().to_owned();

    // 创建channel用于发送热键事件
    let (tx, rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let key_groups = settings_collected.keys_collection;
        let mut hkm = HotkeyManager::new();

        // 注册截屏快捷键
        let tx_sc = tx.clone();
        let hotkey_sc = hkm.register(key_groups[0].vkey, &key_groups[0].mod_keys, move || {
            tx_sc.send("sc_unchecked").unwrap();
        });
        match hotkey_sc {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey for sc."),
        };

        // 注册钉图快捷键
        let tx_pin = tx.clone();
        let hotkey_pin = hkm.register(key_groups[1].vkey, &key_groups[1].mod_keys, move || {
            tx_pin.send("pin").unwrap();
        });
        match hotkey_pin {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey 2."),
        };

        // 注册设置快捷键
        let tx_conf = tx.clone();
        let hotkey_conf = hkm.register(key_groups[3].vkey, &key_groups[3].mod_keys, move || {
            tx_conf.send("conf").unwrap();
        });
        match hotkey_conf {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey conf."),
        };

        // 注册退出快捷键
        let tx_exit = tx.clone();
        let hotkey_exit = hkm.register(key_groups[2].vkey, &key_groups[2].mod_keys, move || {
            tx_exit.send("exit").unwrap();
        });
        match hotkey_exit {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey 3."),
        };

        // 启动事件循环
        hkm.event_loop();
    });

    (handle, rx)
}

/// 将快捷键配置字符串转换为系统可用的按键组合
///
/// ### 参数
/// - `groups`: 包含按键字符串的结构体
///
/// ### 返回值
/// - `(bool, KeyVkGroups)`: 转换状态和结果
/// - 第一个值表示转换是否成功
/// - 第二个值为转换后的按键组合
pub fn match_keys(groups: &KeyStringGroups) -> (bool, KeyVkGroups) {
    let group1 = &groups.mod_keys;
    let group2 = groups.vkey.as_ref();
    let mut results_mod: [ModKey; 3] = [ModKey::NoRepeat; 3];
    let mut status = true;

    // 转换修饰键(如Ctrl, Alt, Shift等)
    for (i, j) in group1.iter().enumerate() {
        let tmp = match ModKey::from_keyname(j) {
            Ok(mod_key) => mod_key,
            Err(_) => {
                status = false;
                ModKey::NoRepeat
            }
        };
        results_mod[i] = tmp;
    }

    // 转换主键值
    let result_vk = match VKey::from_keyname(group2) {
        Ok(vk_key) => vk_key,
        Err(_) => {
            status = false;
            VKey::OemClear
        }
    };

    // 构建并返回快捷键组合结构体
    let struct_pack = move |x: [ModKey; 3], y: VKey| KeyVkGroups {
        mod_keys: x,
        vkey: y,
    };

    (status, struct_pack(results_mod, result_vk))
}

/// 根据是否需要时间参数来执行不同的命令模式
///
/// ### Arguments
///
/// * `exe_path` - 一个指向可执行文件路径的引用，用于指定需要操作的程序
/// * `is_time` - 一个布尔值，用于决定是否添加时间参数到文件名
/// * `final_path` - 一个指向最终路径的引用，用于指定命令执行后的结果路径
pub fn sc_mode(exe_path: &PathBuf, is_time: bool, final_path: &PathBuf) {
    if is_time {
        operate_exe(&exe_path, "sct", &final_path);
    } else {
        operate_exe(&exe_path, "sc", &final_path);
    }
}
