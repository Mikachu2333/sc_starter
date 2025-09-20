//! SC_Starter 主程序模块
//!
//! 本模块是程序的入口点，负责：
//! - 程序单例检测，防止多开
//! - 初始化程序路径和配置
//! - 创建托盘图标和事件处理
//! - 注册全局快捷键
//! - 启动主事件循环

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 导入各个模块
mod config;
mod file_ops;
mod hotkeys;
mod msgbox;
mod tray;
mod types;
mod window_handle;

use crate::config::*;
use crate::file_ops::*;
use crate::hotkeys::*;
use crate::tray::*;
use crate::types::*;

use single_instance::SingleInstance;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tao::event_loop::EventLoop;
use tray_icon::{MouseButton, TrayIconEvent};
// 新增：用于节流的时间工具
use std::time::{Duration, Instant};

/// 随机生成的GUID，用于程序单例检测
/// 防止程序多开造成快捷键冲突
const PROCESS_ID: &str = "78D83F24ADEC8FAF2E4CC1795F166CE4";

/// 程序主入口函数
///
/// ### 功能流程
/// 1. 检查程序单例运行
/// 2. 初始化程序路径和文件
/// 3. 读取配置文件
/// 4. 创建托盘图标
/// 5. 注册全局快捷键
/// 6. 启动主事件循环
fn main() {
    // 使用系统互斥锁确保程序单例运行，防止多个实例造成快捷键冲突
    let instance = Box::new(SingleInstance::new(PROCESS_ID).unwrap());
    if !instance.is_single() {
        // 检测到已有实例在运行时，显示提示并退出
        msgbox::error_msgbox("Avoid Multiple.", "");
        panic!("Multiple!")
    };

    println!("\n@@@@@@@@@@@@@@@@\n@  SC_Starter  @\n@@@@@@@@@@@@@@@@");
    println!(
        "Version:\t{}\nBuild Time:\t{}",
        PKG_VERSION,
        &PKG_BUILD_TIME[..=18]
    );

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
        conf_path: dir_path.join("config.toml"),
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

    let temp = binding
        .config_dir()
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");
    let self_path = {
        let str_path = std::env::args().collect::<Vec<String>>();
        //println!("{}", str_path[0]);
        PathBuf::from(str_path.first().unwrap())
    };
    set_startup(settings.sundry.auto_start, &temp, &self_path);

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
    let save_path = settings.path.save_path.clone();
    let gui = settings.gui.clone();

    let event_handler = std::thread::spawn(move || {
        // 新增：右键触发长截图的节流配置
        let debounce = Duration::from_millis(800);
        let mut last_right_long = Instant::now() - Duration::from_secs(10);

        while let Ok(event) = event_receiver.recv() {
            // 检查程序是否应该退出
            if !*running_clone.lock().unwrap() {
                break;
            }
            let gui_clone = gui.clone();

            match event {
                TrayIconEvent::DoubleClick { button, .. } => {
                    if button == MouseButton::Left {
                        // 左键双击：触发普通截图
                        let args = [
                            format!(
                                "--comp:{},{}",
                                settings.sundry.comp_level, settings.sundry.scale_level
                            ),
                            save_path_get(&save_path),
                        ]
                        .to_vec();
                        operate_exe(&exe_path, args, gui_clone.clone());
                    }
                }
                // 单击事件处理
                TrayIconEvent::Click {
                    button,
                    button_state,
                    ..
                } => {
                    if button_state == tray_icon::MouseButtonState::Up {
                        if button == MouseButton::Middle {
                            // 中键单击：退出
                            *running_clone.lock().unwrap() = false;
                            operate_exe(&PathBuf::new(), "exit", std::collections::HashMap::new());
                            break;
                        } else if button == MouseButton::Right {
                            // 右键单击：触发截长屏（加入节流，避免短时间内重复触发）
                            let now = Instant::now();
                            if now.duration_since(last_right_long) >= debounce {
                                last_right_long = now;
                                let args = [
                                    "--cap:long".to_string(),
                                    format!(
                                        "--comp:{},{}",
                                        settings.sundry.comp_level, settings.sundry.scale_level
                                    ),
                                    save_path_get(&save_path),
                                ]
                                .to_vec();
                                operate_exe(&exe_path, args, gui_clone.clone());
                            } else {
                                // 忽略抖动/快速重复点击
                            }
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
    event_loop.run(move |_event, _, control_flow| {
        // 一旦托盘事件发生，会通过 UserEvent 唤醒到这里
        if !*running.lock().unwrap() || event_handler.is_finished() {
            // 清理热键线程
            if let Some(handle) = Arc::clone(&handler_hotkeys).lock().unwrap().take() {
                hotkey_exit_tx.send(()).ok(); // 发送退出信号
                if handle.join().is_ok() {
                    println!("Hotkey handler thread terminated.");
                }
            }

            // 清理托盘图标
            let _ = &tray_manager; // 触发 Drop trait 实现

            *control_flow = tao::event_loop::ControlFlow::Exit; // 退出事件循环
        } else {
            *control_flow = tao::event_loop::ControlFlow::Wait;
        }
    });
}
