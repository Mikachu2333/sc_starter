//! 系统托盘管理模块
//!
//! 本模块负责：
//! - 创建和管理系统托盘图标
//! - 创建右键上下文菜单（截图、长截图、退出）
//! - 处理托盘图标事件
//! - 显示程序版本信息

use tray_icon::menu::{Menu, MenuId, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder};

use crate::types::{PKG_VERSION, RES_VERSION};

/// 系统托盘管理器
///
/// 负责管理系统托盘图标的创建、显示、右键菜单和事件处理
pub struct TrayManager {
    /// 托盘图标实例
    tray_icon: tray_icon::TrayIcon,
    /// 截图菜单项 ID
    capture_id: MenuId,
    /// 长截图菜单项 ID
    long_capture_id: MenuId,
    /// 退出菜单项 ID
    exit_id: MenuId,
}

impl TrayManager {
    /// 创建新的托盘管理器实例
    ///
    /// ### 返回值
    /// - `Self`: 初始化完成的托盘管理器实例
    ///
    /// ### 说明
    /// - 从嵌入的图标数据创建256x256的RGBA图标
    /// - 创建包含"截图"、"长截图"、"退出"的右键菜单
    /// - 设置包含程序版本和资源版本的提示文本
    /// - 自动构建并显示托盘图标
    ///
    /// ### Panics
    /// - 如果嵌入的图标数据无效，会panic
    /// - 如果构建托盘图标失败，会panic
    pub fn new(lang: bool) -> Self {
        // 创建托盘图标
        let icon_data = include_bytes!("../logo_raw") as &[u8];
        let icon = Icon::from_rgba(icon_data.to_vec(), 256, 256).expect("Embedded icon is invalid");

        // 创建右键菜单（根据语言设置显示对应文本）
        let menu = Menu::new();
        let menu_capture = MenuItem::new(if lang { "截图" } else { "Capture" }, true, None);
        let menu_long_capture =
            MenuItem::new(if lang { "长截图" } else { "Long Capture" }, true, None);
        let menu_exit = MenuItem::new(if lang { "退出" } else { "Exit" }, true, None);

        menu.append(&menu_capture).unwrap();
        menu.append(&menu_long_capture).unwrap();
        menu.append(&PredefinedMenuItem::separator()).unwrap();
        menu.append(&menu_exit).unwrap();

        // 保存菜单项 ID 用于事件匹配
        let capture_id = menu_capture.id().clone();
        let long_capture_id = menu_long_capture.id().clone();
        let exit_id = menu_exit.id().clone();

        let tray_icon = TrayIconBuilder::new()
            .with_tooltip(format!("SC_Starter v{}\nRES v{}", PKG_VERSION, RES_VERSION))
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build()
            .unwrap();

        Self {
            tray_icon,
            capture_id,
            long_capture_id,
            exit_id,
        }
    }

    /// 获取截图菜单项的 ID
    pub fn capture_id(&self) -> &MenuId {
        &self.capture_id
    }

    /// 获取长截图菜单项的 ID
    pub fn long_capture_id(&self) -> &MenuId {
        &self.long_capture_id
    }

    /// 获取退出菜单项的 ID
    pub fn exit_id(&self) -> &MenuId {
        &self.exit_id
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
