//! 快捷键管理模块
//!
//! 本模块负责：
//! - 注册全局快捷键
//! - 处理快捷键事件
//! - 管理快捷键线程

use crate::file_ops::operate_exe;
use crate::types::*;
use std::sync::mpsc;
use std::thread;
use std::{path::PathBuf, thread::JoinHandle};
use windows_hotkeys::{singlethreaded::HotkeyManager, HotkeyManagerImpl};

/// 设置全局快捷键并返回事件发送器
pub fn set_hotkeys(
    paths: &PathInfos,
    settings_collected: &SettingsCollection,
) -> (JoinHandle<()>, mpsc::Sender<()>) {
    let settings_collected = settings_collected.clone();
    let (exit_tx, exit_rx) = mpsc::channel();

    let exe_path = paths.exe_path.clone();
    let final_path = settings_collected.path.clone();
    let conf_path = paths.conf_path.clone();
    let gui = settings_collected.gui_conf.clone();

    let handle = thread::spawn(move || {
        let key_groups = settings_collected.keys_collection;
        let mut hkm = HotkeyManager::new();

        let exe_path_clone = exe_path.clone();
        let final_path_clone = final_path.clone();
        let gui_clone = gui.clone();

        // 注册截屏快捷键
        let hotkey_sc = hkm.register(
            key_groups.get("screen_capture").unwrap().vkey,
            &key_groups.get("screen_capture").unwrap().mod_keys,
            move || {
                operate_exe(
                    &exe_path_clone,
                    &parms_get( &final_path_clone),
                    &gui_clone,
                );
            },
        );
        if hotkey_sc.is_err() {
            panic!("Failed reg Hotkey sc.");
        };

        let exe_path_clone = exe_path.clone();
        let gui_clone = gui.clone();
        let final_path_clone = final_path.clone();

        // 注册截长屏快捷键
        let hotkey_scl = hkm.register(
            key_groups.get("screen_capture_long").unwrap().vkey,
            &key_groups.get("screen_capture_long").unwrap().mod_keys,
            move || {
                operate_exe(
                    &exe_path_clone,
                    &("--cap:long*".to_string() + &parms_get( &final_path_clone)),
                    &gui_clone,
                );
            },
        );
        if hotkey_scl.is_err() {
            panic!("Failed reg Hotkey sc.");
        };

        let exe_path_clone = exe_path.clone();
        let gui_clone = gui.clone();

        // 注册钉图快捷键
        let hotkey_pin = hkm.register(
            key_groups.get("pin_to_screen").unwrap().vkey,
            &key_groups.get("pin_to_screen").unwrap().mod_keys,
            move || {
                operate_exe(&exe_path_clone, "pin", &gui_clone);
            },
        );
        if hotkey_pin.is_err() {
            panic!("Failed reg Hotkey pin.");
        };

        // 注册设置快捷键
        let hotkey_conf = hkm.register(
            key_groups.get("open_conf").unwrap().vkey,
            &key_groups.get("open_conf").unwrap().mod_keys,
            move || operate_exe(&conf_path, "conf", &String::new()),
        );
        match hotkey_conf {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey conf."),
        };

        // 注册退出快捷键
        let hotkey_exit = hkm.register(
            key_groups.get("exit").unwrap().vkey,
            &key_groups.get("exit").unwrap().mod_keys,
            move || std::process::exit(0),
        );
        match hotkey_exit {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey exit."),
        };

        // 添加消息循环
        while exit_rx.try_recv().is_err() {
            // 处理所有等待的消息
            hkm.handle_hotkey();
        }
    });

    (handle, exit_tx)
}

pub fn parms_get( save_path: &PathBuf) -> String {
    let mut parm: Vec<String> = Vec::new();
    if save_path != &PathBuf::new() {
        parm.push(format!(
            r#"--path:"{}""#,
            save_path.to_str().unwrap().replace("\\", "/")
        ));
    }

    if parm.is_empty() {
        "".to_string()
    } else {
        parm.join("*")
    }
}
