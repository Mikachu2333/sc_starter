use crate::types::{FileExist, PathInfos};
use rust_embed::*;
use std::{
    fs,
    os::windows::{fs::MetadataExt, process::CommandExt},
    path::{Path, PathBuf},
    process::Command,
};

// ScreenCapture.exe v2.2.7 的文件大小
const RES_SIZE: u64 = 8612352;

/// 检查所需文件是否存在及其状态
///
/// ### 参数
/// * `infos` - 包含程序所需的所有路径信息
///
/// ### 返回值
/// 返回一个 `FileExist` 结构体，包含:
/// * `exe_exist` - exe文件是否存在
/// * `exe_latest` - exe文件是否为最新版本
/// * `conf_exist` - 配置文件是否存在
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
        let _ = fs::write(&paths.exe_path, screen_capture_res.data.as_ref());
        println!("Release exe file.");
    }
    if !exists.conf_exist {
        let _ = fs::write(&paths.conf_path, config_res.data.as_ref());
        println!("Release config file.");
        operate_exe(&paths.conf_path, 3, &PathBuf::new());
        operate_exe(Path::new(""), 4, &PathBuf::new());
        operate_exe(Path::new(""), 2, &PathBuf::new());
    } else {
        println!("No need to release.");
    }
}

/// 程序操作控制函数
///
/// ### 参数
/// * `path` - 要操作的程序路径
/// * `mode` - 操作模式:
///     - 0: 运行截屏
///     - 1: 钉图
///     - 2: 退出程序
///     - 3: 打开设置文件
///     - 4: 3秒后提示重启
/// * `dir` - 指定的工作目录路径
///
/// ### Panics
/// * 当 `mode` 参数不在有效范围内时会 panic
/// * 当路径无效或程序无法启动时会显示错误提示
pub fn operate_exe(path: &Path, mode: usize, dir: &PathBuf) {
    match mode {
        0 => {
            let temp = format!("--dir:\"{}\"", dir.to_str().unwrap());
            if dir != &PathBuf::new() {
                //println!("{}", temp);
                let _ = Command::new(path).raw_arg(temp).spawn();
            } else {
                let _ = Command::new(path).spawn();
            }
        }
        1 => {
            let _ = Command::new(path)
                .raw_arg("--pin:clipboard,100,100")
                .spawn();
        }
        2 => {
            println!("Exit");
            let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Exit',3,'Info',64);close()\"").spawn();
            std::process::exit(0)
        }
        3 => {
            match Command::new("notepad.exe").arg(path).spawn() {
                Ok(_) => (),
                Err(_) => {
                    let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Error to open the config file with notepad.',0,'Error',16);close()\"").spawn();
                }
            };
        }
        4 => {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('Please restart the program to apply your custom settings.',0,'Info',64);close()\"").spawn();
            std::process::exit(0);
        }
        _ => panic!("Error arg!"),
    }
}

/// 防止程序文件被删除的监控函数
///
/// ### 参数
/// * `paths` - 包含需要监控的文件路径
///
/// ### 功能
/// * 每5秒检查一次文件是否存在
/// * 如果文件被删除，显示错误信息并终止程序
pub fn avoid_exe_del(paths: &PathInfos) {
    let path1 = paths.conf_path.to_owned();
    let path2 = paths.exe_path.to_owned();
    let handler_check = std::thread::spawn(move || {
        loop {
            if path1.exists() && path2.exists() {
                std::thread::sleep(std::time::Duration::from_secs(5));
            } else {
                break;
            }
        }
        let _ = std::process::Command::new("mshta")
            .raw_arg("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('File not found.',0,'Error',16);close()\"").spawn();
        panic!("File not found.");
    });
    handler_check.join().unwrap();
}
