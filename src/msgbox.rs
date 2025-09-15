use std::os::windows::process::CommandExt;

#[allow(dead_code)]
enum MsgBtnType {
    Ok,
    OkCancel,
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

#[allow(dead_code)]
enum MsgBoxType {
    Error,
    Info,
    Quest,
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

/// If title is empty, it will be set to MsgBoxType
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

/// Displays an information message box with an OK button.
/// If title is empty, it will be set to "Information"
#[allow(dead_code)]
pub fn info_msgbox(msg: impl ToString, title: impl ToString) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Info, MsgBtnType::Ok)
}

/// Displays an error message box with an OK button.
/// If title is empty, it will be set to "Error"
#[allow(dead_code)]
pub fn error_msgbox(msg: impl ToString, title: impl ToString) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Error, MsgBtnType::Ok)
}

/// Displays a warning message box with an OK button.
/// If title is empty, it will be set to "Warning"
#[allow(dead_code)]
pub fn warn_msgbox(msg: impl ToString, title: impl ToString) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Warn, MsgBtnType::Ok)
}

/// Displays a question message box with Yes/No buttons.
/// If title is empty, it will be set to "Question"
#[allow(dead_code)]
pub fn quest_msgbox_yesno(msg: impl ToString, title: impl ToString) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Quest, MsgBtnType::YesNo)
}

/// Displays a question message box with OK/Cancel buttons.
/// If title is empty, it will be set to "Question"
#[allow(dead_code)]
pub fn quest_msgbox_okcancel(msg: impl ToString, title: impl ToString) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Quest, MsgBtnType::OkCancel)
}
