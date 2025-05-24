//! 系统托盘管理模块
//!
//! 本模块负责：
//! - 创建和管理系统托盘图标
//! - 处理托盘图标事件
//! - 显示程序版本信息

use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent};

use crate::types::RES_VERSION;

/// 编译时获取的包版本号
const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 系统托盘管理器
///
/// 负责管理系统托盘图标的创建、显示和事件处理
pub struct TrayManager {
    /// 托盘图标实例
    tray_icon: tray_icon::TrayIcon,
}

impl TrayManager {
    /// 创建新的托盘管理器实例
    ///
    /// ### 返回值
    /// - `Self`: 初始化完成的托盘管理器实例
    ///
    /// ### 说明
    /// - 从嵌入的图标数据创建256x256的RGBA图标
    /// - 设置包含程序版本和资源版本的提示文本
    /// - 自动构建并显示托盘图标
    ///
    /// ### Panics
    /// - 如果嵌入的图标数据无效，会panic
    /// - 如果构建托盘图标失败，会panic
    pub fn new() -> Self {
        // 创建托盘图标
        let icon_data = include_bytes!("../logo_raw") as &[u8];
        let icon = Icon::from_rgba(icon_data.to_vec(), 256, 256).expect("Embedded icon is invalid");

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip(format!("SC_Starter v{}\nRES v{}", PKG_VERSION, RES_VERSION))
            .with_icon(icon)
            .build()
            .unwrap();

        Self { tray_icon }
    }

    /// 启动托盘事件监听
    ///
    /// ### 返回值
    /// - `tray_icon::TrayIconEventReceiver`: 托盘图标事件接收器
    ///
    /// ### 说明
    /// - 返回的接收器可用于监听托盘图标的各种事件
    /// - 调用方需要在事件循环中处理接收到的事件
    /// - 接收器是克隆的，可以安全地在多个地方使用
    pub fn run(&self) -> tray_icon::TrayIconEventReceiver {
        TrayIconEvent::receiver().to_owned()
    }
}

impl Drop for TrayManager {
    /// 清理资源
    ///
    /// ### 说明
    /// - 在对象销毁时自动调用
    /// - 确保托盘图标被正确隐藏和移除
    /// - 防止系统托盘中留下僵尸图标
    fn drop(&mut self) {
        // 确保托盘图标被正确移除
        self.tray_icon.set_visible(false).ok();
    }
}
