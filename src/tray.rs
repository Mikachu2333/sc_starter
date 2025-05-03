use std::path::PathBuf;
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};

use crate::file_ops::operate_exe;

pub struct TrayManager {
    _tray_icon: tray_icon::TrayIcon,
    exe_path: PathBuf,
    save_path: PathBuf,
    time_enabled: bool,
}

impl TrayManager {
    pub fn new(exe_path: PathBuf, save_path: PathBuf, time_enabled: bool) -> Self {
        // 创建托盘图标
        let icon_data = include_bytes!("../logo_raw") as &[u8];
        let icon = Icon::from_rgba(icon_data.to_vec(), 256, 256).expect("Embedded icon is invalid");

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Screen Capture")
            .with_icon(icon)
            .build()
            .unwrap();

        Self {
            _tray_icon: tray_icon,
            exe_path,
            save_path,
            time_enabled,
        }
    }

    pub fn run(&self) -> std::thread::JoinHandle<()> {
        let exe_path = self.exe_path.clone();
        let save_path = self.save_path.clone();
        let time_enabled = self.time_enabled;
        println!("托盘图标已创建");
        std::thread::spawn(move || {
            // 使用循环持续接收事件
            println!("托盘图标已创建，等待事件...");
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                println!("{:?}", event);
            }
        })
    }
}
