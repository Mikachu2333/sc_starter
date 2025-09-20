//! 快捷键管理模块
//!
//! 本模块负责：
//! - 注册全局快捷键
//! - 处理快捷键事件
//! - 管理快捷键线程
//! - 管理启动应用程序的进程状态

use crate::types::*;
use crate::window_handle::{is_process_running, set_window_topmost_by_pid};
use crate::{file_ops::operate_exe, msgbox::error_msgbox};
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
    thread,
    {path::PathBuf, thread::JoinHandle},
};
use windows_hotkeys::{singlethreaded::HotkeyManager, HotkeyManagerImpl};

const T_SEC_1_100: std::time::Duration = std::time::Duration::from_millis(10);
const T_SEC_1_2: std::time::Duration = std::time::Duration::from_millis(500);

/// 设置全局快捷键并返回事件发送器
///
/// ### 参数
/// - `paths`: 程序路径信息
/// - `settings_collected`: 配置集合，包含快捷键设置
///
/// ### 返回值
/// - `(JoinHandle<()>, mpsc::Sender<()>)`: 快捷键线程句柄和退出信号发送器
///
/// ### 功能
/// - 在新线程中注册所有全局快捷键
/// - 创建快捷键事件处理循环
/// - 返回线程控制接口
pub fn set_hotkeys(
    paths: &PathInfos,
    settings_collected: &SettingsCollection,
) -> (JoinHandle<()>, mpsc::Sender<()>) {
    let settings_collected = settings_collected.clone();
    let (exit_tx, exit_rx) = mpsc::channel();

    let comp = settings_collected.sundry.comp_level.to_string();
    let scale = settings_collected.sundry.scale_level.to_string();
    let exe_path = paths.exe_path.clone();
    let save_path = settings_collected.path.save_path.clone();
    let launch = settings_collected.path.launch_app.clone();
    let conf_path = paths.conf_path.clone();
    let gui = settings_collected.gui.clone();

    let handle = thread::spawn(move || {
        let key_groups = settings_collected.keys_collection;
        let mut hkm = HotkeyManager::new();

        let comp_clone = comp.clone();
        let scale_clone = scale.clone();
        let exe_path_clone = exe_path.clone();
        let save_path_clone = save_path.clone();
        let gui_clone = gui.clone();

        // 注册截屏快捷键
        let hotkey_sc = hkm.register(
            key_groups.get("screen_capture").unwrap().vkey,
            &key_groups.get("screen_capture").unwrap().mod_keys,
            move || {
                let args = [
                    format!("--comp:{},{}", comp_clone, scale_clone),
                    save_path_get(&save_path_clone),
                ]
                .to_vec();
                operate_exe(&exe_path_clone, args, gui_clone.clone());
            },
        );
        if hotkey_sc.is_err() {
            let temp = "Failed reg Hotkey sc.";
            error_msgbox(temp, "Register Hotkey Error");
            panic!("{}", &temp);
        };

        if launch.path.exists() {
            // 初始化启动应用程序的进程ID管理
            let launch_pid: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
            // 注册Launch快捷键
            let hotkey_launch = hkm.register(
                key_groups.get("launch_app").unwrap().vkey,
                &key_groups.get("launch_app").unwrap().mod_keys,
                move || {
                    // 检查文件扩展名，判断是否为可执行文件
                    let ext = launch
                        .path
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    let is_executable = ["exe", "bat", "cmd", "com", "msi"].contains(&ext.as_str());

                    if is_executable {
                        // 对于可执行文件，使用进程管理逻辑
                        let current_pid = *launch_pid.lock().unwrap();

                        // 获取启动程序的文件名用于进程检测
                        let process_name = launch
                            .path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("")
                            .to_string();

                        // 检查进程是否仍在运行
                        if current_pid != 0 {
                            unsafe {
                                if is_process_running(&process_name) {
                                    // 进程存在，直接置顶窗口
                                    set_window_topmost_by_pid(current_pid);
                                    return;
                                } else {
                                    // 进程已退出，重置PID
                                    *launch_pid.lock().unwrap() = 0;
                                }
                            }
                        }

                        // 启动新的可执行文件进程
                        let child = if launch.args.join(" ").trim().is_empty() {
                            std::process::Command::new(&launch.path).spawn()
                        } else {
                            std::process::Command::new(&launch.path)
                                .args(&launch.args)
                                .spawn()
                        };

                        // 如果程序启动成功，记录PID并等待窗口创建后置顶
                        if let Ok(child) = child {
                            let pid = child.id();
                            *launch_pid.lock().unwrap() = pid;

                            // 等待程序启动并创建窗口后置顶
                            std::thread::sleep(T_SEC_1_2);
                            unsafe {
                                set_window_topmost_by_pid(pid);
                            }
                        }
                    } else {
                        // 对于非可执行文件，直接通过默认程序或explorer打开
                        // 不进行进程管理，每次都重新打开
                        if launch.args.join(" ").trim().is_empty() {
                            // 使用系统默认程序打开文件
                            let _ = std::process::Command::new("explorer")
                                .arg(&launch.path)
                                .spawn();
                        } else {
                            // 如果有参数，尝试直接执行（可能是特定的打开方式）
                            let _ = std::process::Command::new(&launch.path)
                                .args(&launch.args)
                                .spawn();
                        }
                    }
                },
            );
            if hotkey_launch.is_err() {
                let temp = "Failed reg Hotkey launch.";
                error_msgbox(temp, "Register Hotkey Error");
                panic!("{}", &temp);
            };
        }

        let comp_clone = comp.clone();
        let scale_clone = scale.clone();
        let exe_path_clone = exe_path.clone();
        let gui_clone = gui.clone();
        let save_path_clone = save_path.clone();

        // 注册截长屏快捷键
        let hotkey_scl = hkm.register(
            key_groups.get("screen_capture_long").unwrap().vkey,
            &key_groups.get("screen_capture_long").unwrap().mod_keys,
            move || {
                let args = [
                    "--cap:long".to_string(),
                    format!("--comp:{},{}", comp_clone, scale_clone),
                    save_path_get(&save_path_clone),
                ]
                .to_vec();
                operate_exe(&exe_path_clone, args, gui_clone.clone());
            },
        );
        if hotkey_scl.is_err() {
            let temp = "Failed reg Hotkey scl.";
            error_msgbox(temp, "Register Hotkey Error");
            panic!("{}", &temp);
        };

        let exe_path_clone = exe_path.clone();
        let gui_clone = gui.clone();

        // 注册钉图快捷键
        let hotkey_pin = hkm.register(
            key_groups.get("pin_to_screen").unwrap().vkey,
            &key_groups.get("pin_to_screen").unwrap().mod_keys,
            move || {
                operate_exe(&exe_path_clone, "pin", gui_clone.clone());
            },
        );
        if hotkey_pin.is_err() {
            let temp = "Failed reg Hotkey pin.";
            error_msgbox(temp, "Register Hotkey Error");
            panic!("{}", &temp);
        };

        // 注册设置快捷键
        let hotkey_conf = hkm.register(
            key_groups.get("open_conf").unwrap().vkey,
            &key_groups.get("open_conf").unwrap().mod_keys,
            move || operate_exe(&conf_path, "conf", HashMap::new()),
        );
        if hotkey_conf.is_err() {
            let temp = "Failed reg Hotkey conf.";
            error_msgbox(temp, "Register Hotkey Error");
            panic!("{}", &temp);
        }

        // 注册退出快捷键
        let hotkey_exit = hkm.register(
            key_groups.get("exit").unwrap().vkey,
            &key_groups.get("exit").unwrap().mod_keys,
            move || std::process::exit(0),
        );
        if hotkey_exit.is_err() {
            let temp = "Failed reg Hotkey exit.";
            error_msgbox(temp, "Register Hotkey Error");
            panic!("{}", &temp);
        }

        // 添加消息循环
        while exit_rx.try_recv().is_err() {
            // 处理所有等待的消息
            hkm.handle_hotkey();
            std::thread::sleep(T_SEC_1_100);
        }
    });

    (handle, exit_tx)
}

/// 生成保存路径参数
///
/// ### 参数
/// - `save_path`: 用户配置的保存路径
///
/// ### 返回值
/// - `String`: 格式化后的命令行参数字符串
///   - 如果路径为空，返回空字符串，表示使用默认路径
///   - 否则返回格式化的 --path:"路径" 参数
pub fn save_path_get(save_path: &PathBuf) -> String {
    if save_path == &PathBuf::new() {
        String::new()
    } else {
        format!(
            r#"--path:"{}""#,
            save_path.to_str().unwrap().replace("\\", "/")
        )
    }
}
