#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
extern crate single_instance;
use directories::BaseDirs;
use rust_embed::*;
use single_instance::SingleInstance;
use std::{
    fs::{self, File},
    io::Read,
    os::windows::{fs::MetadataExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{exit, Command},
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

fn unzip_res(paths: &PathInfos, exists: &FileExist) {
    //解压相关资源
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
        operate_exe(&paths.conf_path, 3);
        operate_exe(Path::new(""), 4);
        operate_exe(Path::new(""), 2);
    } else {
        println!("No need to release.");
    }
}

fn operate_exe(path: &Path, mode: usize) {
    // 实际操作程序
    // 0 -> 运行截屏
    // 1 -> 钉图
    // 2 -> 退出
    // 3 -> 打开设置文件
    // 4 -> 3s后提示重启
    // ... -> panic
    match mode {
        0 => {
            let _ = Command::new(path).spawn().unwrap();
        }
        1 => {
            let _ = Command::new(path).arg("--pin:clipboard").spawn().unwrap();
        }
        2 => exit(0),
        3 => {
            let _ = Command::new("explorer.exe").arg(path).spawn().unwrap();
        }
        4 => {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _output: Result<std::process::Child, std::io::Error> = Command::new("mshta.exe")
                .raw_arg(r#"vbscript:Execute("msgbox ""Please Restart SC_Starter"" :close")"#)
                .spawn();
        }
        _ => panic!("Error arg!"),
    }
}

fn set_hotkeys(paths: &PathInfos, key_groups: Vec<KeyVkGroups>) -> JoinHandle<()> {
    //根据配置文件设置快捷键
    let exe_path = paths.exe_path.to_owned();
    let conf_path = paths.conf_path.to_owned();
    thread::spawn(move || {
        let res_path = exe_path.clone();
        let key_groups = key_groups;
        let mut hkm = HotkeyManager::new();

        let hotkey_1 = hkm.register(key_groups[0].vkey, &key_groups[0].mod_keys, move || {
            operate_exe(&exe_path, 0);
        });
        match hotkey_1 {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey 1."),
        };

        let hotkey_2 = hkm.register(key_groups[1].vkey, &key_groups[1].mod_keys, move || {
            operate_exe(&res_path, 1);
        });
        match hotkey_2 {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey 2."),
        }

        let hotkey_3 = hkm.register(key_groups[2].vkey, &key_groups[2].mod_keys, move || {
            operate_exe(Path::new(""), 2);
        });
        match hotkey_3 {
            Ok(_) => (),
            Err(_) => panic!("Failed reg Hotkey 3."),
        }

        hkm.register(key_groups[3].vkey, &key_groups[3].mod_keys, move || {
            operate_exe(&conf_path, 3);
        })
        .unwrap();

        hkm.event_loop();
    })
}

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

fn read_config(conf_path: &PathBuf, default_settings: &[KeyVkGroups]) -> Vec<KeyVkGroups> {
    //读取配置
    let mut f = File::open(conf_path).unwrap();
    let mut full_content = String::new();
    let _ = f.read_to_string(&mut full_content);
    let full_content: Vec<&str> = full_content.split("\n").collect();
    //暴力只取前4行，实际上conf文件后面注释的#号纯粹无用，好看而已（
    //println!("{:?}",&full_content[0..4]);
    let usefull_content: Vec<&str> = full_content[0..4].to_vec();

    //这个闭包接收两个String然后返回一个包装好的KeyStringGroups类型便于下面解析
    let struct_pack = |x: Vec<String>, y: String| KeyStringGroups {
        mod_keys: x,
        vkey: y,
    };

    //4个配置得到4个group，再整合成一个groups
    let mut groups: Vec<KeyStringGroups> = Vec::new();
    for i in usefull_content {
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
            result_groups.push(default_settings[i].clone());
        }
    }

    result_groups
}

fn avoid_exe_del(conf_path: &Path) {
    let path = conf_path.to_owned();
    let handler_check = thread::spawn(move || {
        loop {
            if path.exists() {
                std::thread::sleep(std::time::Duration::from_secs(5));
            } else {
                break;
            }
        }
        panic!("Config file not found.");
    });
    //阻塞运行时间最长的那个（panic无法使整个程序退出，只能退出其所在的线程）
    handler_check.join().unwrap();
}

fn main() {
    // Use CreateMuteA to avoid multi-process
    let instance = Box::new(SingleInstance::new(PROCESS_ID).unwrap());
    if !instance.is_single() {
        panic!("Avoid Multiple.")
    };

    //Init
    let mut path_infos = PathInfos {
        dir_path: PathBuf::from(BaseDirs::new().unwrap().data_local_dir()).join("SC_Starter"),
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
    let default_setting: Vec<KeyVkGroups> = Vec::from([
        KeyVkGroups {
            mod_keys: Vec::from([ModKey::Win, ModKey::Alt]),
            vkey: VKey::V,
        },
        KeyVkGroups {
            mod_keys: Vec::from([ModKey::Win, ModKey::Alt]),
            vkey: VKey::C,
        },
        KeyVkGroups {
            mod_keys: Vec::from([ModKey::Win, ModKey::Ctrl, ModKey::Shift]),
            vkey: VKey::Escape,
        },
        KeyVkGroups {
            mod_keys: Vec::from([ModKey::Ctrl, ModKey::Alt]),
            vkey: VKey::O,
        },
    ]);
    //Read Setting
    let settings = read_config(&path_infos.conf_path, &default_setting);
    for i in &settings {
        println!("Groups: {}", i);
    }
    //Set Hotkeys
    let _handler_hotkeys = set_hotkeys(&path_infos, settings);

    //防止配置文件被删除
    avoid_exe_del(&path_infos.exe_path);
}
