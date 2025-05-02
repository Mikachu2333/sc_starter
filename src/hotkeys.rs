use crate::file_ops::operate_exe;
use crate::types::{KeyStringGroups, KeyVkGroups, PathInfos, SettingsCollection};
use std::thread;
use std::{path::PathBuf, thread::JoinHandle};
use windows_hotkeys::{
    keys::{ModKey, VKey},
    singlethreaded::HotkeyManager,
    HotkeyManagerImpl,
};

/// 根据配置文件设置快捷键
/// ### Arguments
/// * `paths` - 包含程序路径信息的结构体
/// * `settings_collected` - 包含设置信息的结构体
/// ### Returns
/// * `JoinHandle<()>` - 线程句柄
pub fn set_hotkeys(paths: &PathInfos, settings_collected: SettingsCollection) -> JoinHandle<()> {
    let exe_path = paths.exe_path.to_owned();
    let conf_path = paths.conf_path.to_owned();
    let dir = settings_collected.path.to_path_buf();

    thread::spawn(move || {
        let res_path = exe_path.clone();
        let key_groups = settings_collected.keys_collection;
        let mut hkm = HotkeyManager::new();

        // 注册截屏快捷键
        // 触发时执行operate_exe函数，参数mode=0表示截屏操作
        let hotkey_1 = hkm.register(key_groups[0].vkey, &key_groups[0].mod_keys, move || {
            operate_exe(&exe_path, 0, &dir.to_path_buf());
        });
        match hotkey_1 {
            Ok(_) => (),
            Err(_) => {
                operate_exe(&conf_path, 3, &PathBuf::new());
                panic!("Failed reg Hotkey 1.")
            }
        };

        // 注册钉图快捷键
        // 触发时执行operate_exe函数，参数mode=1表示钉图操作
        let hotkey_2 = hkm.register(key_groups[1].vkey, &key_groups[1].mod_keys, move || {
            operate_exe(&res_path, 1, &PathBuf::new());
        });
        match hotkey_2 {
            Ok(_) => (),
            Err(_) => {
                operate_exe(&conf_path, 3, &PathBuf::new());
                panic!("Failed reg Hotkey 2.")
            }
        }

        // 注册退出快捷键
        // 触发时执行operate_exe函数，参数mode=2表示退出操作
        let hotkey_3 = hkm.register(key_groups[2].vkey, &key_groups[2].mod_keys, move || {
            operate_exe(std::path::Path::new(""), 2, &PathBuf::new());
        });
        match hotkey_3 {
            Ok(_) => (),
            Err(_) => {
                operate_exe(&conf_path, 3, &PathBuf::new());
                panic!("Failed reg Hotkey 3.")
            }
        }

        // 注册重启快捷键
        // 触发时执行operate_exe函数，参数mode=3表示重启操作
        let conf_path_clone = conf_path.clone();
        let hotkey_4 = hkm.register(key_groups[3].vkey, &key_groups[3].mod_keys, move || {
            operate_exe(&conf_path, 3, &PathBuf::new());
        });
        match hotkey_4 {
            Ok(_) => (),
            Err(_) => {
                operate_exe(&conf_path_clone, 3, &PathBuf::new());
                panic!("Failed reg Hotkey 4.")
            }
        }

        // 启动事件循环，监听快捷键
        hkm.event_loop();
    })
}

/// 将字符串形式的按键转换为系统快捷键值
/// ### Arguments
/// * `groups` - 包含字符串形式按键的结构体
/// ### Returns
/// * `(bool, KeyVkGroups)` - (转换是否成功的状态, 转换后的快捷键组合)
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
