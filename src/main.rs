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
    // 使用系统互斥锁确保程序单例运行，防止多个实例造成快捷键冲突
    let instance = Box::new(SingleInstance::new(PROCESS_ID).unwrap());
    if !instance.is_single() {
        // 检测到已有实例在运行时，显示提示并退出
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
    let tray_manager = TrayManager::new();

    // 获取托盘图标事件接收器
    let event_receiver = tray_manager.run();

    // 创建事件循环和状态管理
    let event_loop = EventLoop::new();
    let running = Arc::new(Mutex::new(true)); // 程序运行状态标志
    let running_clone = running.clone();

    // 托盘事件处理线程
    // 处理用户与托盘图标的交互，如点击和双击事件
    let exe_path = path_infos.exe_path.clone();
    let save_path = settings.path.clone();
    let time_enabled = settings.time;

    let event_handler = std::thread::spawn(move || {
        while let Ok(event) = event_receiver.recv() {
            // 检查程序是否应该退出
            if !*running_clone.lock().unwrap() {
                break;
            }

            match event {
                // 左键双击：触发截图
                TrayIconEvent::DoubleClick { button, .. } => {
                    if button == MouseButton::Left {
                        sc_mode(&exe_path, time_enabled, &save_path);
                    }
                }
                // 单击事件处理
                TrayIconEvent::Click {
                    button,
                    button_state,
                    ..
                } => {
                    if button_state == MouseButtonState::Down {
                        if button == MouseButton::Right {
                            // 右键单击：退出程序
                            *running_clone.lock().unwrap() = false;
                            operate_exe(&PathBuf::new(), "exit", &PathBuf::new());
                            break;
                        } else {
                            // 左键单击：触发截图
                            sc_mode(&exe_path, time_enabled, &save_path);
                        }
                    }
                }
                _ => {}
            }
        }
    });

    // 设置全局热键并获取事件处理器
    let (handler_hotkeys, hotkey_exit_tx) = set_hotkeys(&path_infos, &settings);
    // 使用 Arc<Mutex<Option<>>> 包装热键处理线程，便于安全地在多线程间共享和清理
    let handler_hotkeys: Arc<Mutex<Option<std::thread::JoinHandle<()>>>> =
        Arc::new(Mutex::new(Some(handler_hotkeys)));

    // 启动文件监控，防止核心文件被删除
    avoid_exe_del(&path_infos);

    // 主事件循环
    // 处理热键事件和程序状态管理
    event_loop.run(move |_event, _, control_flow| {
        // 程序退出处理
        if !*running.lock().unwrap() || event_handler.is_finished() {
            // 清理热键线程
            if let Some(handle) = Arc::clone(&handler_hotkeys).lock().unwrap().take() {
                hotkey_exit_tx.send(()).ok(); // 发送退出信号
                if let Ok(_) = handle.join() {
                    println!("Hotkey handler thread terminated.");
                }
            }

            // 清理托盘图标
            let _ = &tray_manager; // 触发 Drop trait 实现

            *control_flow = tao::event_loop::ControlFlow::Exit; // 退出事件循环
        } else {
            // 继续等待事件
            *control_flow = tao::event_loop::ControlFlow::Wait;
        }
    });
}
