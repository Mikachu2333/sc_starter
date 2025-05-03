#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 导入各个模块
mod config;
mod file_ops;
mod hotkeys;
mod tray;
mod types;

use crate::config::*;
use crate::file_ops::*;
use crate::hotkeys::*;
use crate::tray::*;
use crate::types::*;

use single_instance;
use single_instance::SingleInstance;
use std::{
    os::windows::process::CommandExt,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tao::event_loop::EventLoop;
use tray_icon::MouseButton;
use tray_icon::MouseButtonState;
use tray_icon::TrayIconEvent;

/// 随机生成的GUID，用于程序单例检测
/// 防止程序多开造成快捷键冲突
const PROCESS_ID: &str = "C950E2CF78E7358DC0B2A754D49D298E";

fn main() {
    // 使用 CreateMuteA 确保程序单例运行
    let instance = Box::new(SingleInstance::new(PROCESS_ID).unwrap());
    if !instance.is_single() {
        // 如果检测到程序已运行，弹出提示框并退出
        let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Avoid Multiple.',0,'Error',16);close()\"").spawn();
        panic!("Multiple!")
    };

    // 初始化路径信息
    // 设置程序所需文件的存放路径，包括:
    // 1. 主程序目录 (AppData/Local/SC_Starter)
    // 2. 截图程序路径
    // 3. 配置文件路径
    let binding = directories::BaseDirs::new().unwrap();
    let data_local_dir = binding.data_local_dir();
    let dir_path = PathBuf::from(data_local_dir).join("SC_Starter");
    let path_infos = PathInfos {
        dir_path: dir_path.clone(),
        exe_path: dir_path.join("ScreenCapture.exe"),
        conf_path: dir_path.join("config.ini"),
    };
    println!("{}", &path_infos);

    // 检查必要文件是否存在
    let exist_result = check_res_exist(&path_infos);
    // 根据检查结果解压资源文件
    unzip_res(&path_infos, &exist_result);

    // 读取配置文件
    // 包含快捷键设置和截图保存路径
    let settings = read_config(&path_infos.conf_path);
    print!("{}", &settings);

    // 创建托盘图标管理器
    let tray_manager = TrayManager::new(
        path_infos.exe_path.clone(),
        settings.path.clone(),
        settings.time.clone(),
    );

    // 获取事件接收器
    let event_receiver = tray_manager.run();

    // 创建共享状态
    let running = Arc::new(Mutex::new(true));
    let running_clone = running.clone();

    // 创建事件循环
    let event_loop = EventLoop::new();

    // 托盘事件处理线程
    let exe_path = path_infos.exe_path.clone();
    let save_path = settings.path.clone();
    let time_enabled = settings.time;

    let event_handler = std::thread::spawn(move || {
        while let Ok(event) = event_receiver.recv() {
            if !*running_clone.lock().unwrap() {
                break;
            }

            match event {
                TrayIconEvent::DoubleClick { button, .. } => {
                    if button == MouseButton::Left {
                        sc_mode(&exe_path, time_enabled, &save_path);
                    }
                }
                TrayIconEvent::Click {
                    button,
                    button_state,
                    ..
                } => {
                    if button_state == MouseButtonState::Down {
                        if button == MouseButton::Right {
                            // 退出程序
                            *running_clone.lock().unwrap() = false;
                            operate_exe(&PathBuf::new(), "exit", &PathBuf::new());
                            break;
                        } else {
                            sc_mode(&exe_path, time_enabled, &save_path);
                        }
                    }
                }
                _ => {}
            }
        }
    });

    // 设置系统全局快捷键
    let (handler_hotkeys, hotkey_receiver) = set_hotkeys(&settings);
    let handler_hotkeys = Arc::new(Mutex::new(Some(handler_hotkeys))); // 包装为 Arc + Mutex + Option

    // 启动文件监控
    avoid_exe_del(&path_infos);

    // 运行事件循环
    let exe_path = path_infos.exe_path.clone();
    let save_path = settings.path.clone();
    let time_enabled = settings.time.clone();
    event_loop.run(move |_event, _, control_flow| {
        // 检查热键事件
        if let Ok(hotkey_event) = hotkey_receiver.try_recv() {
            match hotkey_event {
                "sc_unchecked" => sc_mode(&exe_path, time_enabled, &save_path),
                "pin" => operate_exe(&exe_path, "pin", &PathBuf::new()),
                "conf" => operate_exe(&path_infos.conf_path, "conf", &PathBuf::new()),
                "exit" => {
                    *running.lock().unwrap() = false;
                    operate_exe(&PathBuf::new(), "exit", &PathBuf::new());
                }
                _ => {}
            }
        }

        if !*running.lock().unwrap() || event_handler.is_finished() {
            // 取出 handler_hotkeys 并 join
            if let Some(handle) = Arc::clone(&handler_hotkeys).lock().unwrap().take() {
                if let Ok(_) = handle.join() {
                    println!("Hotkey handler thread terminated.");
                }
            }
            // 确保所有资源都被清理
            *control_flow = tao::event_loop::ControlFlow::Exit;
        } else {
            *control_flow = tao::event_loop::ControlFlow::Wait;
        }
    });
}
