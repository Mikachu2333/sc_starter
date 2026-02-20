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
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use tao::event_loop::EventLoop;
use tray_icon::MouseButton;

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
    let instance = match SingleInstance::new(PROCESS_ID) {
        Ok(inst) => Box::new(inst),
        Err(e) => {
            msgbox::error_msgbox(
                format!("Failed to create single instance lock: {}", e),
                "Fatal Error",
                0,
            );
            panic!("SingleInstance creation failed: {}", e);
        }
    };
    if !instance.is_single() {
        // 检测到已有实例在运行时，显示提示并退出
        msgbox::error_msgbox("Avoid Multiple.", "", 2);
        panic!("Multiple!")
    };

    println!("\n@@@@@@@@@@@@@@@@\n@  SC_Starter  @\n@@@@@@@@@@@@@@@@");
    println!(
        "Version:\t{}\nBuild Time:\t{}",
        PKG_VERSION,
        PKG_BUILD_TIME.get(..=18).unwrap_or(PKG_BUILD_TIME)
    );

    // 初始化路径信息
    // 设置程序所需文件的存放路径，包括:
    // 1. 主程序目录 (AppData/Local/SC_Starter)
    // 2. 截图程序路径
    // 3. 配置文件路径
    let binding = match directories::BaseDirs::new() {
        Some(bd) => bd,
        None => {
            msgbox::error_msgbox(
                "Failed to determine base directories (LOCALAPPDATA missing?).",
                "Fatal Error",
                0,
            );
            panic!("BaseDirs::new() returned None");
        }
    };
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

    // 创建托盘图标管理器（含右键菜单：截图、长截图、退出）
    let tray_manager = TrayManager::new(&settings.sundry.lang);

    // 获取菜单项 ID（用于事件匹配）
    let capture_id = tray_manager.capture_id().clone();
    let long_capture_id = tray_manager.long_capture_id().clone();
    let exit_id = tray_manager.exit_id().clone();

    // 创建事件循环和退出通知代理
    let event_loop = EventLoop::new();
    let proxy = event_loop.create_proxy();

    // 程序运行状态标志（原子操作，线程安全）
    let running = Arc::new(AtomicBool::new(true));

    // 准备事件处理所需的数据
    let exe_path = path_infos.exe_path.clone();
    let save_path = settings.path.save_path.clone();
    let gui = settings.gui.clone();
    let comp_level = settings.sundry.comp_level;
    let scale_level = settings.sundry.scale_level;

    // 托盘事件和菜单事件统一处理线程
    // 使用轮询方式处理托盘图标点击和右键菜单事件
    let running_event = running.clone();
    let proxy_event = proxy.clone();
    let _event_handler = std::thread::spawn(move || {
        let tray_receiver = tray_icon::TrayIconEvent::receiver().to_owned();
        let menu_receiver = tray_icon::menu::MenuEvent::receiver().to_owned();

        while running_event.load(Ordering::SeqCst) {
            // 处理托盘图标事件（左键双击截图，单次左键不响应）
            while let Ok(tray_icon::TrayIconEvent::DoubleClick { button, .. }) =
                tray_receiver.try_recv()
            {
                if button == MouseButton::Left {
                    let args = vec![
                        format!("--comp:{},{}", comp_level, scale_level),
                        save_path_get(&save_path),
                    ];
                    let exe = exe_path.clone();
                    let g = gui.clone();
                    std::thread::spawn(move || {
                        operate_exe(&exe, args, g);
                    });
                }
            }

            // 处理右键菜单事件
            while let Ok(event) = menu_receiver.try_recv() {
                if event.id == capture_id {
                    // 菜单：截图
                    let args = vec![
                        format!("--comp:{},{}", comp_level, scale_level),
                        save_path_get(&save_path),
                    ];
                    let exe = exe_path.clone();
                    let g = gui.clone();
                    std::thread::spawn(move || {
                        operate_exe(&exe, args, g);
                    });
                } else if event.id == long_capture_id {
                    // 菜单：长截图
                    let args = vec![
                        "--cap:long".to_string(),
                        format!("--comp:{},{}", comp_level, scale_level),
                        save_path_get(&save_path),
                    ];
                    let exe = exe_path.clone();
                    let g = gui.clone();
                    std::thread::spawn(move || {
                        operate_exe(&exe, args, g);
                    });
                } else if event.id == exit_id {
                    // 菜单：退出
                    println!("Menu: Exit requested");
                    running_event.store(false, Ordering::SeqCst);
                    proxy_event.send_event(()).ok();
                    break;
                }
            }

            std::thread::sleep(Duration::from_millis(10));
        }
    });

    // 设置全局热键并获取控制句柄
    let (handler_hotkeys, hotkey_exit_tx) =
        set_hotkeys(&path_infos, &settings, running.clone(), proxy.clone());
    let handler_hotkeys: Mutex<Option<std::thread::JoinHandle<()>>> =
        Mutex::new(Some(handler_hotkeys));

    // 启动文件监控，防止核心文件被删除
    let _file_monitor_running = avoid_exe_del(&path_infos);

    // 包装 tray_manager 以便在退出时显式 drop
    let mut tray_manager = Some(tray_manager);

    // 主事件循环
    event_loop.run(move |event, _, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Wait;

        // 检查是否收到退出信号（来自菜单退出或热键退出）
        let should_exit = matches!(event, tao::event::Event::UserEvent(()));

        if should_exit {
            // 通知所有线程退出
            running.store(false, Ordering::SeqCst);

            // 清理热键线程
            if let Some(handle) = handler_hotkeys.lock().unwrap().take() {
                hotkey_exit_tx.send(()).ok();
                if handle.join().is_ok() {
                    println!("Hotkey handler thread terminated.");
                }
            }

            // 显式 drop 托盘管理器以清理系统托盘图标
            if let Some(tm) = tray_manager.take() {
                drop(tm);
                println!("Tray manager cleaned up.");
            }

            *control_flow = tao::event_loop::ControlFlow::Exit;
        }
    });
}
