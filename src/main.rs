#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use directories::BaseDirs;
use fslock::LockFile;
use rust_embed::*;
use sha3::{Digest, Sha3_256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    process::{exit, Command},
    thread::{self, JoinHandle},
};
use windows_hotkeys::{
    keys::{ModKey, VKey},
    singlethreaded::HotkeyManager,
    HotkeyManagerImpl,
};

fn check_res_exist() -> (bool, bool, PathBuf) {
    // 测试是否存在或需要替换exe文件
    // (if_res_exist, if_res_latest, res_path)
    let mut path_for_res =
        PathBuf::from(BaseDirs::new().unwrap().data_local_dir()).join("SC_starter");
    if !path_for_res.exists() {
        let _ = fs::create_dir_all(&path_for_res);
    };

    let lock_path = path_for_res.join("lock_file");
    if !lock_path.exists() {
        let _ = File::create(&lock_path).unwrap().write_all(b"Lock");
    }

    path_for_res.push("ScreenCapture.exe");
    let if_exist_exe = path_for_res.exists();

    let mut if_latest = false;
    if if_exist_exe {
        let read_sha3 = calc_sha3_256(&path_for_res);
        if read_sha3 == "0dfedae82300fca4bb5f75b8b083cffcc42518006ccd0b50eaea06ad6a433e74" {
            if_latest = true;
        } else {
            if_latest = false;
        }
    }

    return (if_exist_exe, if_latest, path_for_res);
}

fn calc_sha3_256(file_path: &PathBuf) -> String {
    let mut file = File::open(file_path).expect("无法打开文件");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).expect("无法读取文件内容");

    let mut hasher = Sha3_256::new();
    hasher.update(&buffer);
    let result = hasher.finalize();

    format!("{:x}", result)
}

fn unzip_res(res_path: &PathBuf) {
    #[derive(Embed)]
    #[folder = "res/"]
    struct Asset;
    let screen_capture_res =
        Asset::get("ScreenCapture.exe").expect("Error read embedded res file.");

    let _ = fs::write(res_path, screen_capture_res.data.as_ref());

    println!("Finish release Exe.");
}

fn set_hotkeys(exe_path: &PathBuf) -> JoinHandle<()> {
    let exe_path = exe_path.to_owned().clone();
    thread::spawn(move || {
        let exe_path = exe_path.clone();
        let res_path = exe_path.clone();
        let mut hkm = HotkeyManager::new();
        hkm.register(VKey::V, &[ModKey::Win, ModKey::Alt], move || {
            Command::new(&exe_path).spawn().unwrap();
        })
        .unwrap();

        hkm.register(VKey::C, &[ModKey::Win, ModKey::Alt], move || {
            let _ = Command::new(&res_path)
                .arg("--pin:clipboard")
                .spawn()
                .unwrap();
        })
        .unwrap();

        hkm.register(VKey::Escape, &[ModKey::Win, ModKey::Alt], move || {
            exit(0);
        })
        .unwrap();

        hkm.event_loop();
    })
}

fn main() {
    // 检测是否存在并释放
    let (if_res_exist, if_res_latest, res_path) = check_res_exist();
    let dir_path = res_path.parent().unwrap().to_path_buf();

    let mut lock_file = LockFile::open(&dir_path.join("lock_file")).unwrap();
    lock_file.lock().unwrap();

    if !if_res_exist {
        println!("Exe Not exist.");
        unzip_res(&res_path);
    } else if !if_res_latest {
        println!("Exe exist, Not latest.");
        unzip_res(&res_path);
    } else {
        println!("Exe exist, Latest.");
    }

    let handler = set_hotkeys(&res_path);
    handler.join().unwrap();
}
