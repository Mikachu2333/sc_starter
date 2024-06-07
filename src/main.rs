#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use directories::BaseDirs;
use rust_embed::*;
use std::{
    fs::{self, File},
    io::Read,
    os::windows::fs::MetadataExt,
    path::PathBuf,
    process::{exit, Command},
    thread::{self, JoinHandle},
};
use windows_hotkeys::{
    keys::{ModKey, VKey},
    singlethreaded::HotkeyManager,
    HotkeyManagerImpl,
};

#[derive(Clone, Copy, Debug)]
struct FileExist {
    exe_exist: bool,
    exe_latest: bool,
    conf_exist: bool,
}

#[derive(Clone)]
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

fn check_res_exist(infos: &PathInfos) -> FileExist {
    // 测试是否存在或需要替换exe文件
    let mut files_exist = FileExist {
        exe_exist: false,
        exe_latest: false,
        conf_exist: false,
    };
    if !infos.dir_path.exists() {
        let _ = fs::create_dir_all(&infos.dir_path);
    };

    files_exist.exe_exist = infos.exe_path.exists();
    files_exist.conf_exist = infos.conf_path.exists();

    if files_exist.exe_exist {
        files_exist.exe_exist = true;
        if check_exe_latest(&infos.exe_path) {
            files_exist.exe_latest = true;
        } else {
            files_exist.exe_latest = false;
        }
    }
    files_exist
}

fn check_exe_latest(file_path: &PathBuf) -> bool {
    let in_size = file_path.metadata().unwrap().file_size();
    let original_size: u64 = 3905024;
    if in_size == original_size {
        return true;
    } else {
        return false;
    }
}

fn unzip_res(paths: &PathInfos, exists: &FileExist) {
    #[derive(Embed)]
    #[folder = "res/"]
    struct Asset;
    let screen_capture_res =
        Asset::get("ScreenCapture.exe").expect("Error read embedded EXE res file.");
    let config_res = Asset::get("config.txt").expect("Error read embedded Config res file.");

    if (!exists.exe_exist) || (!exists.exe_latest) {
        let _ = fs::write(&paths.exe_path, screen_capture_res.data.as_ref());
    }
    if !exists.conf_exist {
        let _ = fs::write(&paths.conf_path, config_res.data.as_ref());
    }

    println!("Finish release Exe.");
}

fn operate_exe(path: &PathBuf, mode: u8) {
    match mode {
        0 => {
            exit(0);
        }
        1 => {
            Command::new(&path).spawn().unwrap();
        }
        2 => {
            Command::new(&path).arg("--pin:clipboard").spawn().unwrap();
        }
        3 => {
            Command::new("explorer.exe").arg(&path).spawn().unwrap();
        }
        _ => (),
    }
}

fn set_hotkeys(
    exe_path: &PathBuf,
    conf_path: &PathBuf,
    key_groups: Vec<KeyVkGroups>,
) -> JoinHandle<()> {
    let exe_path = exe_path.clone();
    let conf_path = conf_path.clone();
    let key_groups = key_groups.clone();
    thread::spawn(move || {
        let res_path = exe_path.clone();
        let key_groups = key_groups;
        let mut hkm = HotkeyManager::new();
        hkm.register(key_groups[0].vkey, &key_groups[0].mod_keys, move || {
            operate_exe(&exe_path, 1);
        })
        .unwrap();

        hkm.register(key_groups[1].vkey, &key_groups[1].mod_keys, move || {
            operate_exe(&res_path, 2);
        })
        .unwrap();

        hkm.register(key_groups[2].vkey, &key_groups[2].mod_keys, move || {
            operate_exe(&PathBuf::new(), 0);
        })
        .unwrap();

        hkm.register(key_groups[3].vkey, &key_groups[3].mod_keys, move || {
            operate_exe(&conf_path, 3);
        })
        .unwrap();

        hkm.event_loop();
    })
}

fn match_keys(groups: KeyStringGroups) -> (bool, KeyVkGroups) {
    let group1 = groups.mod_keys;
    let group2 = groups.vkey;
    let mut results_mod: Vec<ModKey> = Vec::new();

    for i in &group1 {
        let tmp = match ModKey::from_keyname(i) {
            Ok(mod_key) => mod_key,
            Err(_) => ModKey::NoRepeat,
        };
        results_mod.push(tmp);
    }

    let result_vk = match VKey::from_keyname(&group2) {
        Ok(vk_key) => vk_key,
        Err(_) => VKey::OemClear,
    };

    let mut success = true;
    for i in &results_mod {
        if *i == ModKey::NoRepeat {
            success = false;
        }
    }

    if result_vk == VKey::OemClear {
        success = false;
    }

    let struct_pack = |x: Vec<ModKey>, y: VKey| {
        let tmp = KeyVkGroups {
            mod_keys: x,
            vkey: y,
        };
        tmp
    };
    let temp: KeyVkGroups = struct_pack(results_mod, result_vk);
    return (success, temp);
}

fn read_config(conf_path: &PathBuf) -> Vec<KeyVkGroups> {
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
    //读取配置
    let mut f = File::open(conf_path).unwrap();
    let mut full_content = String::new();
    let _ = f.read_to_string(&mut full_content);
    let full_content: Vec<&str> = full_content.split("\n").collect();
    let usefull_content: Vec<&str> = full_content[..4].to_vec();

    let struct_pack = |x: Vec<String>, y: String| {
        let tmp = KeyStringGroups {
            mod_keys: x,
            vkey: y,
        };
        tmp
    };

    let mut groups: Vec<KeyStringGroups> = Vec::new();
    for i in usefull_content {
        let sum_keys: Vec<String> = i.split("=").map(String::from).collect();
        let mod_keys: Vec<String> = sum_keys[0].split("+").map(String::from).collect();
        groups.push(struct_pack(mod_keys, sum_keys[1].clone()));
    }

    let mut result_groups: Vec<KeyVkGroups> = Vec::new();
    let mut count: usize = 0;
    for i in groups {
        let (status, result) = match_keys(i);
        //println!("{} {:?}",&status,&result);
        if status {
            result_groups.push(result);
        } else {
            result_groups.push(default_setting[count].clone());
        }
        count += 1;
    }

    result_groups
}

fn main() {
    //Init
    let mut path_infos = PathInfos {
        dir_path: PathBuf::from(BaseDirs::new().unwrap().data_local_dir()).join("SC_starter"),
        exe_path: PathBuf::new(),
        conf_path: PathBuf::new(),
    };
    path_infos.exe_path = path_infos.dir_path.join("ScreenCapture.exe");
    path_infos.conf_path = path_infos.dir_path.join("config.txt");
    //println!("{}", &path_infos);

    let exist_result = check_res_exist(&path_infos);
    //println!("{:?}", &exist_result);
    unzip_res(&path_infos, &exist_result);

    //Read Setting
    let settings = read_config(&path_infos.conf_path);
    println!("{:?}", &settings);
    //Set Hotkeys
    let handler = set_hotkeys(&path_infos.exe_path, &path_infos.conf_path, settings);
    handler.join().unwrap();
}
