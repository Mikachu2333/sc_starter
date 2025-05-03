use std::path::PathBuf;
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};

pub struct TrayManager {
    #[allow(dead_code)]
    tray_icon: tray_icon::TrayIcon,
    #[allow(dead_code)]
    exe_path: PathBuf,
    #[allow(dead_code)]
    save_path: PathBuf,
    #[allow(dead_code)]
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
            tray_icon,
            exe_path,
            save_path,
            time_enabled,
        }
    }

    // 简化run方法，只返回事件接收器
    pub fn run(&self) -> tray_icon::TrayIconEventReceiver {
        TrayIconEvent::receiver().to_owned()
    }
}

impl Drop for TrayManager {
    fn drop(&mut self) {
        // 确保托盘图标被正确移除
        self.tray_icon.set_visible(false).ok();
    }
}
