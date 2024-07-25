#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use core::time;
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
    //直接通过程序size判断是否为最新版，此size为2.1.10.0版本
    let original_size: u64 = 3920896;
    if in_size == original_size {
        return true;
    } else {
        return false;
    }
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
    }
    if !exists.conf_exist {
        let _ = fs::write(&paths.conf_path, config_res.data.as_ref());
    }

    println!("Finish release Exe.");
}

fn operate_exe(path: &PathBuf, mode: usize) {
    //实际操作程序进行调用
    match mode {
        0 => {
            Command::new(&path).spawn().unwrap();
        }
        1 => {
            Command::new(&path).arg("--pin:clipboard").spawn().unwrap();
        }
        2 => {
            exit(0);
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
    //根据配置文件设置快捷键
    //因为检测的延迟性，至少要多出1s以免注册失败
    thread::sleep(time::Duration::from_secs(2));
    let exe_path = exe_path.clone();
    let conf_path = conf_path.clone();
    let key_groups = key_groups.clone();
    thread::spawn(move || {
        let res_path = exe_path.clone();
        let key_groups = key_groups;
        let mut hkm = HotkeyManager::new();

        hkm.register(key_groups[0].vkey, &key_groups[0].mod_keys, move || {
            operate_exe(&exe_path, 0);
        })
        .unwrap();

        hkm.register(key_groups[1].vkey, &key_groups[1].mod_keys, move || {
            operate_exe(&res_path, 1);
        })
        .unwrap();

        hkm.register(key_groups[2].vkey, &key_groups[2].mod_keys, move || {
            operate_exe(&PathBuf::new(), 2);
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
    //将字符串类型快捷键从转换为快捷键组合，并返回状态与快捷键组
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
    let result_vk = match VKey::from_keyname(&group2.to_ascii_uppercase()) {
        Ok(vk_key) => vk_key,
        Err(_) => VKey::OemClear,
    };

    let mut status = true;
    for i in &results_mod {
        if *i == ModKey::NoRepeat {
            status = false;
        }
    }

    if result_vk == VKey::OemClear {
        status = false;
    }

    let struct_pack = |x: Vec<ModKey>, y: VKey| {
        let tmp = KeyVkGroups {
            mod_keys: x,
            vkey: y,
        };
        tmp
    };
    let temp: KeyVkGroups = struct_pack(results_mod, result_vk);
    return (status, temp);
}

fn read_config(conf_path: &PathBuf, default_settings: &Vec<KeyVkGroups>) -> Vec<KeyVkGroups> {
    //读取配置
    let mut f = File::open(conf_path).unwrap();
    let mut full_content = String::new();
    let _ = f.read_to_string(&mut full_content);
    let full_content: Vec<&str> = full_content.split("\n").collect();
    //暴力只取前4行，实际上conf文件后面注释的#号纯粹无用，好看而已（
    let usefull_content: Vec<&str> = full_content[..4].to_vec();

    //这个闭包接收两个String然后返回一个包装好的KeyStringGroups类型便于下面解析
    let struct_pack = |x: Vec<String>, y: String| {
        let tmp = KeyStringGroups {
            mod_keys: x,
            vkey: y,
        };
        tmp
    };

    //4个配置得到4个group，再整合成一个groups
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
            result_groups.push(default_settings[count].clone());
        }
        count += 1;
    }

    result_groups
}

fn get_time(config_dir: &PathBuf) -> PathBuf {
    //获取当前系统秒数并创建文件，同时如果有旧的删除旧的，为实现单实例做准备
    let seconds = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(n) => String::from("TIME") + n.as_secs().to_string().as_str(),
        Err(_) => String::from("0"),
    };
    let time_check_file = config_dir.join(&seconds);
    if !time_check_file.exists() {
        match fs::File::create(&time_check_file) {
            Ok(_) => true,
            Err(_) => panic!("No permissions."),
        };
    } else {
        exit(-1);
    }

    let mut time_file_nums: Vec<u64> = Vec::new();
    for entry in fs::read_dir(config_dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap();
        if name.starts_with("TIME") {
            time_file_nums.push((name.split_at(4).1).parse::<u64>().unwrap());
        }
    }
    while time_file_nums.len() > 1 {
        if time_file_nums[0] < time_file_nums[1] {
            let _ = fs::remove_file(config_dir.join(format!("{}{}", "TIME", time_file_nums[0])))
                .unwrap();
            time_file_nums.remove(0);
        } else {
            let _ = fs::remove_file(config_dir.join(format!("{}{}", "TIME", time_file_nums[1])))
                .unwrap();
            time_file_nums.remove(1);
        }
    }

    time_check_file
}

fn avoid_multiple(check_file: &PathBuf) -> JoinHandle<()> {
    //避免多开，仍然不是很完善……
    let file_path = check_file.clone();
    let handle = thread::spawn(move || loop {
        if file_path.exists() {
            thread::sleep(time::Duration::from_secs(1))
        } else {
            exit(-1);
        }
    });
    handle
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

    //防止重复
    let time_file_path = get_time(&path_infos.dir_path);
    let _time_handler = avoid_multiple(&time_file_path);
    //下面这行加了反而不行……
    //time_handler.join().unwrap();

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
    println!("{:?}", &settings);
    //Set Hotkeys
    let handler = set_hotkeys(&path_infos.exe_path, &path_infos.conf_path, settings);
    handler.join().unwrap();
}
