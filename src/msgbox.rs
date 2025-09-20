//! Windows 消息框模块
//!
//! 本模块提供了显示各种类型消息框的功能，包括：
//! - 信息提示框
//! - 错误提示框  
//! - 警告提示框
//! - 询问对话框（支持Yes/No和OK/Cancel按钮）

use std::os::windows::process::CommandExt;

/// 消息框按钮类型枚举
/// 定义消息框中可用的按钮组合
#[allow(dead_code)]
enum MsgBtnType {
    /// 仅显示"确定"按钮
    Ok,
    /// 显示"确定"和"取消"按钮
    OkCancel,
    /// 显示"是"和"否"按钮
    YesNo,
}
impl MsgBtnType {
    fn to_u8(&self) -> u8 {
        match self {
            MsgBtnType::Ok => 0,
            MsgBtnType::OkCancel => 1,
            MsgBtnType::YesNo => 4,
        }
    }
}

/// 消息框图标类型枚举
/// 定义消息框中显示的图标和默认标题
#[allow(dead_code)]
enum MsgBoxType {
    /// 错误图标（红色X）
    Error,
    /// 信息图标（蓝色i）
    Info,
    /// 问题图标（蓝色?）
    Quest,
    /// 警告图标（黄色!）
    Warn,
}
impl MsgBoxType {
    fn to_u8(&self) -> u8 {
        match self {
            MsgBoxType::Error => 16,
            MsgBoxType::Quest => 32,
            MsgBoxType::Warn => 48,
            MsgBoxType::Info => 64,
        }
    }
    fn to_string(&self) -> String {
        match self {
            MsgBoxType::Error => "Error".to_string(),
            MsgBoxType::Quest => "Question".to_string(),
            MsgBoxType::Warn => "Warning".to_string(),
            MsgBoxType::Info => "Information".to_string(),
        }
    }
}

/// 显示消息框的底层实现函数
///
/// ### 参数
/// - `msg`: 消息内容
/// - `title`: 消息框标题（如果为空，将使用消息类型作为标题）
/// - `msgtype`: 消息框图标类型
/// - `btntype`: 按钮类型
///
/// ### 返回值
/// - `i32`: 用户点击按钮的返回码
///
/// ### 功能
/// - 转义特殊字符防止脚本注入
/// - 限制消息长度避免显示问题
/// - 使用mshta调用JavaScript显示原生Windows消息框
/// - 自动设置默认标题
fn raw_msgbox(
    msg: impl ToString,
    title: impl ToString,
    msgtype: MsgBoxType,
    btntype: MsgBtnType,
) -> i32 {
    let context = |x: String| {
        let mut result = x
            .replace("\\", "\\\\")
            .replace("'", "\\'")
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
            .replace("\0", "\\0")
            .replace("\x08", "\\b")
            .replace("\x0C", "\\f");

        if result.len() > 400 {
            result.truncate(400);
            result.push_str("...");
        }

        result
    };
    let msg = msg.to_string();
    let title = {
        let temp = title.to_string();
        if temp.is_empty() {
            msgtype.to_string()
        } else {
            temp
        }
    };

    let parm = format!("\"javascript:var sh=new ActiveXObject('WScript.Shell'); sh.Popup('{}',{},'{}',{});close()\"",context(msg),btntype.to_u8(),context(title),msgtype.to_u8());

    let result = std::process::Command::new("mshta")
        .raw_arg(parm)
        .output()
        .unwrap();
    result.status.code().unwrap()
}

/// 显示信息消息框
///
/// ### 参数
/// - `msg`: 消息内容
/// - `title`: 消息框标题（如果为空，将显示"Information"）
///
/// ### 功能
/// - 显示蓝色信息图标
/// - 仅包含"确定"按钮
/// - 适用于向用户提供信息反馈
#[allow(dead_code)]
pub fn info_msgbox(msg: impl ToString, title: impl ToString) {
    let _ = raw_msgbox(msg, title, MsgBoxType::Info, MsgBtnType::Ok);
}

/// 显示错误消息框
///
/// ### 参数
/// - `msg`: 错误消息内容
/// - `title`: 消息框标题（如果为空，将显示"Error"）
///
/// ### 功能
/// - 显示红色错误图标
/// - 仅包含"确定"按钮
/// - 适用于显示错误信息和异常情况
#[allow(dead_code)]
pub fn error_msgbox(msg: impl ToString, title: impl ToString) {
    let _ = raw_msgbox(msg, title, MsgBoxType::Error, MsgBtnType::Ok);
}

/// 显示警告消息框
///
/// ### 参数
/// - `msg`: 警告消息内容
/// - `title`: 消息框标题（如果为空，将显示"Warning"）
///
/// ### 返回值
/// - `i32`: 用户操作的返回码（通常为确定按钮）
///
/// ### 功能
/// - 显示黄色警告图标
/// - 仅包含"确定"按钮
/// - 适用于显示警告信息和注意事项
#[allow(dead_code)]
pub fn warn_msgbox(msg: impl ToString, title: impl ToString) {
    let _ = raw_msgbox(msg, title, MsgBoxType::Warn, MsgBtnType::Ok);
}

/// 显示Yes/No询问对话框
///
/// ### 参数
/// - `msg`: 询问消息内容
/// - `title`: 消息框标题（如果为空，将显示"Question"）
///
/// ### 返回值
/// - `i32`: 用户选择的返回码
///   - 6: 用户点击"是"
///   - 7: 用户点击"否"
///
/// ### 功能
/// - 显示蓝色问号图标
/// - 包含"是"和"否"按钮
/// - 适用于需要用户确认的二选一场景
#[allow(dead_code)]
pub fn quest_msgbox_yesno(msg: impl ToString, title: impl ToString) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Quest, MsgBtnType::YesNo)
}

/// 显示OK/Cancel询问对话框
///
/// ### 参数
/// - `msg`: 询问消息内容
/// - `title`: 消息框标题（如果为空，将显示"Question"）
///
/// ### 返回值
/// - `i32`: 用户选择的返回码
///   - 1: 用户点击"确定"
///   - 2: 用户点击"取消"
///
/// ### 功能
/// - 显示蓝色问号图标
/// - 包含"确定"和"取消"按钮
/// - 适用于操作确认场景
#[allow(dead_code)]
pub fn quest_msgbox_okcancel(msg: impl ToString, title: impl ToString) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Quest, MsgBtnType::OkCancel)
}
