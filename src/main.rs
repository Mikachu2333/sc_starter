#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// 导入各个模块
mod config;
mod file_ops;
mod hotkeys;
//mod tray;
mod types;

use crate::config::*;
use crate::file_ops::*;
use crate::hotkeys::*;
//use crate::tray::*;
use crate::types::*;

use single_instance;
use single_instance::SingleInstance;
use std::{os::windows::process::CommandExt, path::PathBuf};

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
    print!("{}", settings);

    /*
    // 创建并运行托盘图标
    let tray_manager = TrayManager::new(
        path_infos.exe_path.clone(),
        settings.path.clone(),
        settings.time,
    );
    let tray_handler = tray_manager.run();
     */

    // 设置系统全局快捷键
    let handler_hotkeys = set_hotkeys(&path_infos, settings);

    // 等待热键线程结束
    handler_hotkeys.join().unwrap();

    // 等待托盘线程结束
    //tray_handler.join().unwrap();

    // 启动文件监控
    avoid_exe_del(&path_infos);
}
