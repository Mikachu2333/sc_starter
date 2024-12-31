#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate single_instance;
use native_dialog::MessageDialog;
use rust_embed::*;
use single_instance::SingleInstance;
use std::{
    fs::{self, File},
    io::Read,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    process::Command,
    thread::{self, JoinHandle},
};
use windows_hotkeys::{
    keys::{ModKey, VKey},
    singlethreaded::HotkeyManager,
    HotkeyManagerImpl,
};

//随机的GUID，用于防止多开
const PROCESS_ID: &str = "2E94A7BAE3864EEBA3FCC7AB758C2112";
//此size为2.1.10.0版本
const RES_SIZE: u64 = 3920896;

#[derive(Clone, Copy, Debug)]
struct FileExist {
    exe_exist: bool,
    exe_latest: bool,
    conf_exist: bool,
}

#[derive(Clone, Debug)]
struct SettingsCollection {
    keys_collection: Vec<KeyVkGroups>,
    path: PathBuf,
}

#[derive(Clone, Debug)]
struct PathInfos {
    dir_path: PathBuf,
    exe_path: PathBuf,
    conf_path: PathBuf,
}
impl std::fmt::Display for PathInfos {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "dir: {:?}\nexe: {:?}\nconf: {:?}\n",
            self.dir_path, self.exe_path, self.conf_path
        )
    }
}
#[derive(Clone)]
struct KeyStringGroups {
    mod_keys: Vec<String>,
    vkey: String,
}
#[derive(Clone, Debug)]
struct KeyVkGroups {
    mod_keys: Vec<ModKey>,
    vkey: VKey,
}
impl std::fmt::Display for KeyVkGroups {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "mod keys: {:?}  vkey: {}", self.mod_keys, self.vkey)
    }
}

fn check_res_exist(infos: &PathInfos) -> FileExist {
    // 测试是否存在或需要替换exe文件
    let mut files_exist = FileExist {
        exe_exist: false,
        exe_latest: false,
        conf_exist: false,
    };
    if !infos.dir_path.exists() {
        //如果不存在则创建文件夹避免以后出现error
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

fn check_exe_latest(file_path: &Path) -> bool {
    let in_size = file_path.metadata().unwrap().file_size();
    //直接通过程序size判断是否为最新版
    in_size == RES_SIZE
}

///解压相关资源
fn unzip_res(paths: &PathInfos, exists: &FileExist) {
    #[derive(Embed)]
    #[folder = "res/"]
    struct Asset;
    let screen_capture_res =
        Asset::get("ScreenCapture.exe").expect("Error read embedded EXE res file.");
    let config_res = Asset::get("config.txt").expect("Error read embedded Config res file.");

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

/// 实际操作程序
///
/// 0 -> 运行截屏         1 -> 钉图
///
/// 2 -> 退出             3 -> 打开设置文件
///
/// 4 -> 3s后提示重启     ... -> panic
fn operate_exe(path: &Path, mode: usize, dir: &PathBuf) {
    match mode {
        0 => {
            let temp = format!("--dir:\"{}\"", dir.to_str().unwrap());
            if dir != &PathBuf::new() {
                println!("{}", temp);
                let _ = Command::new(path).arg(temp).spawn();
            } else {
                let _ = Command::new(path).spawn();
            }
        }
        1 => {
            let _ = Command::new(path).arg("--pin:clipboard").spawn();
        }
        2 => {
            println!("Exit");
            let _ = MessageDialog::new()
                .set_title("Exit")
                .set_text("Exit.")
                .set_type(native_dialog::MessageType::Info)
                .reset_owner()
                .show_alert();
            std::process::exit(0)
        }
        3 => {
            match Command::new("notepad.exe").arg(path).spawn() {
                Ok(_) => (),
                Err(_) => {
                    let _ = MessageDialog::new()
                        .set_title("Error")
                        .set_text("Error to open the setting file with notepad.")
                        .set_type(native_dialog::MessageType::Error)
                        .reset_owner()
                        .show_alert();
                }
            };
        }
        4 => {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _ = MessageDialog::new()
                .set_title("Info")
                .set_text("Please restart the program to apply your custom settings.")
                .set_type(native_dialog::MessageType::Info)
                .reset_owner()
                .show_alert();
        }
        _ => panic!("Error arg!"),
    }
}

///根据配置文件设置快捷键
fn set_hotkeys(paths: &PathInfos, settings_collected: SettingsCollection) -> JoinHandle<()> {
    let exe_path = paths.exe_path.to_owned();
    let conf_path = paths.conf_path.to_owned();
    let dir = settings_collected.path.clone();
    thread::spawn(move || {
        let res_path = exe_path.clone();
        let key_groups = settings_collected.keys_collection;
        let mut hkm = HotkeyManager::new();

        //截屏
        let hotkey_1 = hkm.register(key_groups[0].vkey, &key_groups[0].mod_keys, move || {
            operate_exe(&exe_path, 0, &dir);
        });
        match hotkey_1 {
            Ok(_) => (),
            Err(_) => {
                operate_exe(&conf_path, 3, &PathBuf::new());
                panic!("Failed reg Hotkey 1.")
            }
        };

        //钉图
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

        //退出
        let hotkey_3 = hkm.register(key_groups[2].vkey, &key_groups[2].mod_keys, move || {
            operate_exe(Path::new(""), 2, &PathBuf::new());
        });
        match hotkey_3 {
            Ok(_) => (),
            Err(_) => {
                operate_exe(&conf_path, 3, &PathBuf::new());
                panic!("Failed reg Hotkey 3.")
            }
        }

        //3s重启
        let conf_path_clone = conf_path.clone();
        let hotkey_4 = hkm.register(key_groups[3].vkey, &key_groups[3].mod_keys, move || {
            operate_exe(&conf_path.clone(), 3, &PathBuf::new());
        });
        match hotkey_4 {
            Ok(_) => (),
            Err(_) => {
                operate_exe(&conf_path_clone, 3, &PathBuf::new());
                panic!("Failed reg Hotkey 4.")
            }
        }

        hkm.event_loop();
    })
}

///将string匹配为vk值
fn match_keys(groups: KeyStringGroups) -> (bool, KeyVkGroups) {
    //将字符串类型快捷键从转换为快捷键组合，并返回状态与快捷键组
    let group1 = groups.mod_keys;
    let group2 = groups.vkey.as_ref();
    let mut results_mod: Vec<ModKey> = Vec::new();
    let mut status = true;

    for i in &group1 {
        let tmp = match ModKey::from_keyname(i) {
            Ok(mod_key) => mod_key,
            Err(_) => {
                status = false;
                ModKey::NoRepeat
            }
        };
        results_mod.push(tmp);
    }

    let result_vk = match VKey::from_keyname(group2) {
        Ok(vk_key) => vk_key,
        Err(_) => {
            status = false;
            VKey::OemClear
        }
    };

    let struct_pack = move |x: Vec<ModKey>, y: VKey| KeyVkGroups {
        mod_keys: x,
        vkey: y,
    };

    (status, struct_pack(results_mod, result_vk))
}

///读取配置，出问题了就从default里面取
fn read_config(conf_path: &PathBuf, default_settings: &SettingsCollection) -> SettingsCollection {
    //读取配置
    let mut f = File::open(conf_path).unwrap();
    let mut full_content = String::new();
    let _ = f.read_to_string(&mut full_content);
    let full_content: Vec<&str> = full_content.split("\n").collect();
    //暴力只取前5行，实际上conf文件后面注释的#号纯粹无用，好看而已（
    //println!("{:?}",&full_content[0..4]);
    //println!("{:?}",&full_content[5]);
    let useful_content: Vec<&str> = full_content[0..4].to_vec();
    let useful_path = {
        let temp = full_content[4].replace("\\", "/").replace("//", "/");
        temp.trim_matches(['\\', '/', '\n', '\r', '"', '\'', ' ', '\t'])
            .to_string()
    };

    //这个闭包接收两个String然后返回一个包装好的KeyStringGroups类型便于下面解析
    let struct_pack = |x: Vec<String>, y: String| KeyStringGroups {
        mod_keys: x,
        vkey: y,
    };

    //4个配置得到4个group，再整合成一个groups
    let mut groups: Vec<KeyStringGroups> = Vec::new();
    for i in useful_content {
        let sum_keys: Vec<String> = i.split("=").map(String::from).collect();
        let mod_keys: Vec<String> = sum_keys[0].split("+").map(String::from).collect();
        //println!("{:?}{:?}",sum_keys,mod_keys);
        groups.push(struct_pack(mod_keys, sum_keys[1].clone()));
    }

    let mut result_groups: Vec<KeyVkGroups> = Vec::new();
    for (i, j) in groups.into_iter().enumerate() {
        let (status, result) = match_keys(j);
        //println!("status: {} {:?}", &status, &result);
        if status {
            result_groups.push(result);
        } else {
            result_groups.push(default_settings.keys_collection[i].clone());
        }
    }

    let path_result: PathBuf = match &useful_path[..] {
        "&" => PathBuf::new(),
        "@" => directories::UserDirs::new()
            .unwrap()
            .desktop_dir()
            .unwrap()
            .to_path_buf(),
        "*" => directories::UserDirs::new()
            .unwrap()
            .picture_dir()
            .unwrap()
            .to_path_buf(),
        x => {
            let temp = PathBuf::from(x);
            if !temp.exists() {
                default_settings.path.clone()
            } else {
                temp
            }
        }
    };

    SettingsCollection {
        keys_collection: result_groups,
        path: path_result,
    }
}

///避免exe文件被故意删除导致崩溃
fn avoid_exe_del(paths: &PathInfos) {
    let path1 = paths.conf_path.to_owned();
    let path2 = paths.exe_path.to_owned();
    let handler_check = thread::spawn(move || {
        loop {
            if path1.exists() && path2.exists() {
                std::thread::sleep(std::time::Duration::from_secs(5));
            } else {
                break;
            }
        }
        let _ = MessageDialog::new()
            .set_title("Error")
            .set_text("File not found")
            .set_type(native_dialog::MessageType::Error)
            .reset_owner()
            .show_alert();
        panic!("File not found.");
    });
    //阻塞运行时间最长的那个（panic无法使整个程序退出，只能退出其所在的线程）
    handler_check.join().unwrap();
}

fn main() {
    // Use CreateMuteA to avoid multi-process
    let instance = Box::new(SingleInstance::new(PROCESS_ID).unwrap());
    if !instance.is_single() {
        let _ = MessageDialog::new()
            .set_title("Wanring")
            .set_text("Avoid Multiple.")
            .set_type(native_dialog::MessageType::Warning)
            .reset_owner()
            .show_alert();
        panic!("Multiple")
    };

    //Init
    let mut path_infos = PathInfos {
        dir_path: PathBuf::from(directories::BaseDirs::new().unwrap().data_local_dir())
            .join("SC_Starter"),
        exe_path: PathBuf::new(),
        conf_path: PathBuf::new(),
    };
    path_infos.exe_path = path_infos.dir_path.join("ScreenCapture.exe");
    path_infos.conf_path = path_infos.dir_path.join("config.txt");
    //println!("{}", &path_infos);

    let exist_result = check_res_exist(&path_infos);
    //println!("{:?}", &exist_result);
    unzip_res(&path_infos, &exist_result);

    //Default
    let default_key_setting: Vec<KeyVkGroups> = Vec::from([
        KeyVkGroups {
            //PrintScreen
            mod_keys: Vec::from([ModKey::Win, ModKey::Alt, ModKey::Ctrl]),
            vkey: VKey::P,
        },
        KeyVkGroups {
            //Pin
            mod_keys: Vec::from([ModKey::Win, ModKey::Alt, ModKey::Ctrl]),
            vkey: VKey::C,
        },
        KeyVkGroups {
            //Exit
            mod_keys: Vec::from([ModKey::Win, ModKey::Ctrl, ModKey::Shift]),
            vkey: VKey::Escape,
        },
        KeyVkGroups {
            //OpenSettings
            mod_keys: Vec::from([ModKey::Win, ModKey::Alt, ModKey::Ctrl]),
            vkey: VKey::O,
        },
    ]);
    let default_setting = SettingsCollection {
        keys_collection: default_key_setting,
        path: PathBuf::new(),
    };

    //Read Setting
    let settings = read_config(&path_infos.conf_path, &default_setting);
    for i in &settings.keys_collection {
        println!("Groups: {}", i);
    }
    println!("Dir: {:?}", &settings.path);
    //Set Hotkeys
    let _handler_hotkeys = set_hotkeys(&path_infos, settings);

    //防止程序被删除
    avoid_exe_del(&path_infos);
}
