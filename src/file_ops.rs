//! 文件操作模块
//! 
//! 本模块负责：
//! - 检查和保护核心文件
//! - 处理文件操作请求
//! - 执行外部程序

use crate::types::{FileExist, PathInfos};
use rust_embed::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::{
    fs,
    os::windows::{fs::MetadataExt, process::CommandExt},
    path::{Path, PathBuf},
    process::Command,
};

// ScreenCapture.exe v2.2.13 的文件大小
const RES_SIZE: u64 = 8613888;

/// 检查所需文件是否存在及其状态
/// 
/// ### 参数
/// - `infos`: 包含程序所需的所有路径信息
/// 
/// ### 返回值
/// - `FileExist`: 包含文件状态的结构体
/// 
/// ### 说明
/// - 检查exe文件是否存在
/// - 检查exe文件是否为最新版本
/// - 检查配置文件是否存在
pub fn check_res_exist(infos: &PathInfos) -> FileExist {
    let mut files_exist = FileExist {
        exe_exist: false,
        exe_latest: false,
        conf_exist: false,
    };

    if !infos.dir_path.exists() {
        match fs::create_dir_all(&infos.dir_path) {
            Ok(_) => (),
            Err(_) => panic!("Error"),
        };
    };

    files_exist.exe_exist = infos.exe_path.exists();
    files_exist.conf_exist = infos.conf_path.exists();

    if files_exist.exe_exist {
        files_exist.exe_exist = true;
        files_exist.exe_latest = check_exe_latest(&infos.exe_path);
    }
    files_exist
}

/// 检查exe文件是否为最新版本
/// 通过对比文件大小判断
fn check_exe_latest(file_path: &Path) -> bool {
    let in_size = file_path.metadata().unwrap().file_size();
    in_size == RES_SIZE
}

/// 嵌入资源文件的结构体
#[derive(Embed)]
#[folder = "res/"]
struct Asset;

/// 解压资源文件
///
/// ### 参数
/// * `paths` - 包含所有需要的路径信息
/// * `exists` - 文件存在状态的检查结果
///
/// ### 功能
/// * 如果exe不存在或不是最新版本，释放exe文件
/// * 如果配置文件不存在，释放配置文件并执行初始化操作
pub fn unzip_res(paths: &PathInfos, exists: &FileExist) {
    let screen_capture_res =
        Asset::get("ScreenCapture.exe").expect("Error read embedded EXE res file.");
    let config_res = Asset::get(paths.conf_path.file_name().unwrap().to_str().unwrap())
        .expect("Error read embedded Config res file.");

    if (!exists.exe_exist) || (!exists.exe_latest) {
        let _ = fs::write(&paths.exe_path, screen_capture_res.data.as_ref())
            .expect("Error write EXE file.");
        println!("Release exe file.");
    }
    if !exists.conf_exist {
        let _ = fs::write(&paths.conf_path, config_res.data.as_ref())
            .expect("Error write config file.");
        println!("Release config file.");
        operate_exe(&paths.conf_path, "conf", &PathBuf::new());
        operate_exe(Path::new(""), "restart", &PathBuf::new());
    } else {
        println!("No need to release.");
    }
}

/// 程序操作控制函数
/// 
/// ### 参数
/// - `path`: 要操作的程序路径
/// - `mode`: 操作模式
/// - `save_path`: 指定的保存路径
/// 
/// ### 操作模式
/// - `sc`: 普通截屏
/// - `sct`: 带时间戳的截屏
/// - `pin`: 钉图功能
/// - `exit`: 退出程序
/// - `conf`: 打开配置
/// - `restart`: 程序重启
pub fn operate_exe(path: &Path, mode: &str, save_path: &PathBuf) {
    match mode {
        "sc" => {
            let temp = format!("--dir:\"{}\"", save_path.to_str().unwrap());
            if save_path != &PathBuf::new() {
                //println!("{}", temp);
                let _ = Command::new(path).raw_arg(temp).spawn();
            } else {
                let _ = Command::new(path).spawn();
            }
        }
        "sct" => {
            let temp = format!(
                "--path:\"{}\"",
                save_path
                    .join(get_current_time())
                    .to_str()
                    .unwrap()
                    .replace("\\", "\\\\")
            );
            println!("{}", temp);
            let _ = Command::new(path).raw_arg(temp).spawn();
        }
        "pin" => {
            let _ = Command::new(path).raw_arg("--pin:clipboard").spawn();
        }
        "exit" => {
            println!("Exit");
            let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Exit',3,'Info',64);close()\"").spawn();
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
        _ => panic!("Error arg!"),
    }
}

/// 监控并保护核心文件
/// 
/// ### 参数
/// - `paths`: 包含所有需要保护的文件路径
/// 
/// ### 功能
/// - 持续监控核心文件状态
/// - 文件丢失时自动恢复
/// - 恢复失败时提示用户
pub fn avoid_exe_del(paths: &PathInfos) {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // 创建两个通道
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
}

fn get_current_time() -> String {
    let now = std::time::SystemTime::now();
    let datetime = chrono::DateTime::<chrono::Local>::from(now);
    format!("{}.png", datetime.format("%Y-%m-%d@%H-%M-%S"))
}
