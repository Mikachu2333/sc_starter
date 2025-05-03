//! 快捷键管理模块
//! 
//! 本模块负责：
//! - 注册全局快捷键
//! - 处理快捷键事件
//! - 管理快捷键线程

use crate::file_ops::operate_exe;
use crate::types::{KeyStringGroups, KeyVkGroups, PathInfos, SettingsCollection};
use std::thread;
use std::{path::PathBuf, thread::JoinHandle};
use windows_hotkeys::{
    keys::{ModKey, VKey},
    singlethreaded::HotkeyManager,
    HotkeyManagerImpl,
};

/// 根据配置设置全局快捷键
/// 
/// ### 参数
/// - `paths`: 包含程序路径信息的结构体
/// - `settings_collected`: 包含快捷键设置的结构体
/// 
/// ### 返回值
/// - `JoinHandle<()>`: 快捷键监听线程的句柄
/// 
/// ### 功能
/// - 注册四组全局快捷键
/// - 设置对应的处理函数
/// - 启动事件循环
pub fn set_hotkeys(paths: &PathInfos, settings_collected: SettingsCollection) -> JoinHandle<()> {
    let exe_path = paths.exe_path.to_owned();
    let conf_path = paths.conf_path.to_owned();
    let save_path = settings_collected.path.to_path_buf();

    thread::spawn(move || {
        let res_path = exe_path.clone();
        let key_groups = settings_collected.keys_collection;
        let mut hkm = HotkeyManager::new();

        // 注册截屏快捷键
        // 触发时执行operate_exe函数
        let hotkey_sc = hkm.register(key_groups[0].vkey, &key_groups[0].mod_keys, move || {
            if settings_collected.time {
                operate_exe(&res_path, "sct", &save_path);
            } else {
                operate_exe(&res_path, "sc", &save_path);
            }
        });
        match hotkey_sc {
            Ok(_) => (),
            Err(_) => {
                panic!("Failed reg Hotkey for sc.")
            }
        };

        // 注册钉图快捷键
        // 触发时执行operate_exe函数，参数mode=1表示钉图操作
        let res_path = exe_path.clone();
        let hotkey_pin = hkm.register(key_groups[1].vkey, &key_groups[1].mod_keys, move || {
            operate_exe(&res_path, "pin", &PathBuf::new());
        });
        match hotkey_pin {
            Ok(_) => (),
            Err(_) => {
                panic!("Failed reg Hotkey 2.")
            }
        }

        // 注册设置快捷键
        // 触发时执行operate_exe函数，参数
        let hotkey_conf = hkm.register(key_groups[3].vkey, &key_groups[3].mod_keys, move || {
            operate_exe(&conf_path, "conf", &PathBuf::new());
        });
        match hotkey_conf {
            Ok(_) => (),
            Err(_) => {
                panic!("Failed reg Hotkey conf.")
            }
        }

        // 注册退出快捷键
        // 触发时执行operate_exe函数，参数mode=2表示退出操作
        let hotkey_exit = hkm.register(key_groups[2].vkey, &key_groups[2].mod_keys, move || {
            operate_exe(std::path::Path::new(""), "exit", &PathBuf::new());
        });
        match hotkey_exit {
            Ok(_) => (),
            Err(_) => {
                panic!("Failed reg Hotkey 3.")
            }
        }

        // 启动事件循环，监听快捷键
        hkm.event_loop();
    })
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
