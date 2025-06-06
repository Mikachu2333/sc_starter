//! 文件操作模块
//!
//! 本模块负责：
//! - 检查和保护核心文件
//! - 处理文件操作请求
//! - 执行外部程序
//! - 监控文件状态并自动恢复

use crate::types::{FileExist, PathInfos, RES_HASH};
use rust_embed::*;
use std::{
    collections::HashMap,
    fs,
    os::windows::process::CommandExt,
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

/// 检查所需文件是否存在及其状态
///
/// ### 参数
/// - `infos`: 包含程序所需的所有路径信息
///
/// ### 返回值
/// - `FileExist`: 包含文件状态的结构体，包括：
///   - `exe_exist`: exe文件是否存在
///   - `exe_latest`: exe文件是否为最新版本
///   - `conf_exist`: 配置文件是否存在
///
/// ### 说明
/// - 如果目录不存在会自动创建
/// - 检查exe文件是否存在
/// - 检查exe文件是否为最新版本（通过MD5校验）
/// - 检查配置文件是否存在
pub fn check_res_exist(infos: &PathInfos) -> FileExist {
    let mut files_exist = FileExist::default();

    if !infos.dir_path.exists() {
        match fs::create_dir_all(&infos.dir_path) {
            Ok(_) => (),
            Err(_) => panic!("Error"),
        };
    };

    files_exist.exe_exist = infos.exe_path.exists();
    files_exist.conf_exist = infos.conf_path.exists();

    if files_exist.exe_exist {
        files_exist.exe_latest = check_exe_latest(&infos.exe_path);
    }
    files_exist
}

/// 检查exe文件是否为最新版本
///
/// ### 参数
/// - `file_path`: 要检查的exe文件路径
///
/// ### 返回值
/// - `bool`: 如果文件MD5哈希值与预期的RES_HASH匹配，返回true
///
/// ### 说明
/// - 使用Windows系统的certutil工具计算哈希值
/// - 通过CREATE_NO_WINDOW标志隐藏命令行窗口
/// - 将计算结果与内置的RES_HASH常量进行比较
fn check_exe_latest(file_path: &Path) -> bool {
    let hash = std::process::Command::new("certutil")
        .arg("-hashfile")
        .arg(file_path)
        .arg("SHA1")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW - 隐藏命令行窗口
        .output()
        .expect("Failed to execute command");
    let hash_value = {
        let original = String::from_utf8_lossy(&hash.stdout);
        let line = original.lines().skip(1).next().unwrap();
        line.trim().to_ascii_uppercase()
    };
    RES_HASH.to_ascii_uppercase() == hash_value
}

/// 嵌入资源文件的结构体
///
/// 使用rust_embed将res/目录下的文件嵌入到二进制文件中
#[derive(Embed)]
#[folder = "res/"]
struct Asset;

/// 解压并释放资源文件
///
/// ### 参数
/// * `paths` - 包含所有需要的路径信息
/// * `exists` - 文件存在状态的检查结果
///
/// ### 功能
/// * 如果exe不存在或不是最新版本，释放exe文件到指定位置
/// * 如果配置文件不存在，释放配置文件并执行初始化操作
/// * 首次释放配置文件后会自动打开配置文件并提示重启程序
pub fn unzip_res(paths: &PathInfos, exists: &FileExist) {
    let screen_capture_res = Asset::get(paths.exe_path.file_name().unwrap().to_str().unwrap())
        .expect("Error read embedded EXE res file.");
    let config_res = Asset::get(paths.conf_path.file_name().unwrap().to_str().unwrap())
        .expect("Error read embedded Config res file.");

    if (!exists.exe_exist) || (!exists.exe_latest) {
        let _ = fs::write(&paths.exe_path, screen_capture_res.data.as_ref())
            .expect("Error write EXE file.");
        println!("EXE: Release exe file.");
    } else {
        println!("EXE: No need to release.");
    }
    if !exists.conf_exist {
        let _ = fs::write(&paths.conf_path, config_res.data.as_ref())
            .expect("Error write config file.");
        println!("CONF: Release config file.");
        operate_exe(&paths.conf_path, "conf", HashMap::new());
        operate_exe(Path::new(""), "restart", HashMap::new());
    } else {
        println!("CONF: No need to release.");
    }
}

/// 程序操作控制函数
///
/// ### 参数
/// - `path`: 要操作的程序路径
/// - `mode`: 操作模式字符串
/// - `gui`: GUI相关参数的HashMap，包含normal和long模式的参数
///
/// ### 操作模式
/// - `pin`: 启动钉图功能，从剪贴板获取图像并钉在屏幕上
/// - `exit`: 退出程序，显示退出消息后终止进程
/// - `conf`: 使用记事本打开配置文件进行编辑
/// - `restart`: 显示重启提示消息并退出程序
/// - 其他包含参数的模式: 执行截屏相关操作
///   - 如果包含'*'，会按'*'分割参数
///   - 根据是否包含"long"选择对应的GUI参数
pub fn operate_exe(path: &Path, mode: &str, gui: HashMap<String, String>) {
    match mode {
        "pin" => {
            let _ = Command::new(path).arg("--pin:clipboard").spawn();
        }
        "exit" => {
            println!("Preparing to exit...");
            // 使用异步方式显示消息，避免阻塞
            let _ = std::process::Command::new("mshta")
                .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Exit',1,'Info',64);close()\"")
                .spawn();

            // 给一个短暂的延时让消息显示出来
            std::thread::sleep(std::time::Duration::from_millis(100));
            std::process::exit(0)
        }
        "conf" => {
            match Command::new("notepad.exe").arg(path).spawn() {
                Ok(_) => (),
                Err(_) => {
                    let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Error to open the config file with notepad.',0,'Error',16);close()\"").spawn();
                }
            };
        }
        "restart" => {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Please restart the program to apply your custom settings.',3,'Info',64);close()\"").spawn();
            std::process::exit(0);
        }
        parm => {
            println!("parm: {:?}\narg: {:?}\n", parm, gui.clone());
            if parm.contains('*') {
                // 包含多个参数，按'*'分割
                let temp = parm.split('*').map(String::from);
                let _ = Command::new(path)
                    .args(temp)
                    .arg({
                        if parm.contains("long") {
                            gui.get("long").unwrap()
                        } else {
                            gui.get("normal").unwrap()
                        }
                    })
                    .spawn();
            } else {
                // 单个参数
                let _ = Command::new(path)
                    .arg({
                        if parm.contains("long") {
                            gui.get("long").unwrap()
                        } else {
                            gui.get("normal").unwrap()
                        }
                    })
                    .spawn();
            }
        }
    }
}

/// 监控并保护核心文件
///
/// ### 参数
/// - `paths`: 包含所有需要保护的文件路径
///
/// ### 返回值
/// - `Arc<AtomicBool>`: 线程运行状态的原子布尔值，可用于外部控制监控线程的停止
///
/// ### 功能
/// - 启动后台监控线程，每5秒检查一次核心文件状态
/// - 当检测到exe文件或配置文件丢失时，自动尝试恢复
/// - 使用双线程架构：监控线程负责检测，处理线程负责恢复
/// - 恢复成功后继续监控，恢复失败则终止程序
/// - 通过mpsc通道进行线程间通信，确保线程安全
pub fn avoid_exe_del(paths: &PathInfos) -> Arc<AtomicBool> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // 创建两个通道：文件丢失通知和恢复结果通知
    let (tx_lost, rx_lost) = mpsc::channel();
    let (tx_recovered, rx_recovered) = mpsc::channel();

    // 创建监控线程
    let paths_clone = paths.clone();
    thread::spawn(move || {
        while r.load(Ordering::SeqCst) {
            if !paths_clone.conf_path.exists() || !paths_clone.exe_path.exists() {
                // 发送文件丢失信号
                let _ = tx_lost.send(());
                // 等待恢复完成信号
                match rx_recovered.recv() {
                    Ok(true) => {
                        // 恢复成功，继续下一轮循环
                        println!("Files recovered, resuming monitoring...");
                    }
                    _ => {
                        // 恢复失败或通道关闭，退出线程
                        panic!("Critical files missing and recovery failed!");
                    }
                }
            }
            thread::sleep(std::time::Duration::from_secs(5));
        }
    });

    // 创建处理线程
    let paths_clone = paths.clone();
    let r2 = running.clone();
    thread::spawn(move || {
        while let Ok(()) = rx_lost.recv() {
            println!("Attempting to recover files...");

            // 文件丢失时尝试恢复
            let files_status = check_res_exist(&paths_clone);
            unzip_res(&paths_clone, &files_status);

            // 重新检查恢复结果
            let final_status = check_res_exist(&paths_clone);
            if final_status.exe_exist && final_status.conf_exist && final_status.exe_latest {
                // 恢复成功，发送成功信号
                let _ = tx_recovered.send(true);
            } else {
                // 恢复失败，终止程序
                running.store(false, Ordering::SeqCst);
                let _ = tx_recovered.send(false);
            }
        }
    });

    r2
}
