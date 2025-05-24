use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};

use crate::types::RES_VERSION;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct TrayManager {
    tray_icon: tray_icon::TrayIcon,
}

impl TrayManager {
    pub fn new() -> Self {
        // 创建托盘图标
        let icon_data = include_bytes!("../logo_raw") as &[u8];
        let icon = Icon::from_rgba(icon_data.to_vec(), 256, 256).expect("Embedded icon is invalid");

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip(format!("SC_Starter v{}\nRES v{}", PKG_VERSION,RES_VERSION))
            .with_icon(icon)
            .build()
            .unwrap();

        Self {
            tray_icon,
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
