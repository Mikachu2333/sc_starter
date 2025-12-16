//! Windows message box module
//!
//! Calls User32.dll MessageBoxExW directly to show:
//! - Information dialogs
//! - Error dialogs
//! - Warning dialogs
//! - Question dialogs (Yes/No, OK/Cancel)

use std::{
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

static PROCESS_NAME: &str = std::env!("CARGO_PKG_NAME");

#[allow(clippy::upper_case_acronyms)]
type HWND = isize;
#[allow(clippy::upper_case_acronyms)]
type LPCWSTR = *const u16;
#[allow(clippy::upper_case_acronyms)]
type UINT = u32;
#[allow(clippy::upper_case_acronyms)]
type WORD = u16;
#[allow(clippy::upper_case_acronyms)]
type WPARAM = usize;
#[allow(clippy::upper_case_acronyms)]
type LPARAM = isize;
#[allow(clippy::upper_case_acronyms)]
type BOOL = i32;

const MB_SYSTEMMODAL: UINT = 0x1000;
const MB_SETFOREGROUND: UINT = 0x10000;
const WM_CLOSE: UINT = 0x0010;

#[link(name = "User32")]
extern "system" {
    fn MessageBoxExW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: UINT,
        wLanguageId: WORD,
    ) -> i32;
    fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND;
    fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
}

/// Button combinations for message boxes
#[allow(dead_code)]
enum MsgBtnType {
    /// Only the OK button
    Ok,
    /// OK and Cancel buttons
    OkCancel,
    /// Yes and No buttons
    YesNo,
}
impl MsgBtnType {
    fn to_u32(&self) -> UINT {
        match self {
            MsgBtnType::Ok => 0x0000,
            MsgBtnType::OkCancel => 0x0001,
            MsgBtnType::YesNo => 0x0004,
        }
    }
}

/// Icon styles for message boxes and their default titles
#[allow(dead_code)]
enum MsgBoxType {
    /// Error icon (red X)
    Error,
    /// Information icon (blue i)
    Info,
    /// Question icon (blue ?)
    Quest,
    /// Warning icon (yellow !)
    Warn,
}
impl MsgBoxType {
    fn to_u32(&self) -> UINT {
        match self {
            MsgBoxType::Error => 0x0010,
            MsgBoxType::Quest => 0x0020,
            MsgBoxType::Warn => 0x0030,
            MsgBoxType::Info => 0x0040,
        }
    }
}
impl std::fmt::Display for MsgBoxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MsgBoxType::Error => "Error",
            MsgBoxType::Quest => "Question",
            MsgBoxType::Warn => "Warning",
            MsgBoxType::Info => "Info",
        };
        write!(f, "{}", s)
    }
}

fn normalize_text(text: impl ToString) -> String {
    let result = text.to_string().replace("\r\n", "\n").replace('\r', "\n");
    result.trim().to_string()
}

fn to_wide(text: impl ToString) -> Vec<u16> {
    let text = text.to_string();
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn spawn_timeout_closer(title: Vec<u16>, timeout: u32, timed_out: Arc<AtomicBool>) {
    if timeout == 0 {
        return;
    }

    thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(timeout as u64));
        unsafe {
            let hwnd = FindWindowW(ptr::null(), title.as_ptr());
            if hwnd != 0 {
                timed_out.store(true, Ordering::SeqCst);
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
    });
}

/// Core message box implementation
///
/// ### Parameters
/// - `msg`: Message text
/// - `title`: Dialog title; falls back to message type name when empty
/// - `msgtype`: Icon style
/// - `btntype`: Button combination
/// - `timeout`: Auto-close timeout in seconds (0 means no timeout)
///
/// ### Returns
/// - `i32`: Button result code; returns -1 when closed by timeout
fn raw_msgbox(
    msg: impl ToString,
    title: impl ToString,
    msgtype: MsgBoxType,
    btntype: MsgBtnType,
    timeout: u32,
) -> i32 {
    let msg = normalize_text(msg);
    let title = {
        let t = normalize_text(title);
        let original = if t.is_empty() { msgtype.to_string() } else { t };
        format!(
            "{} [{}] {}",
            original,
            PROCESS_NAME,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        )
    };

    let text_w = to_wide(&msg);
    let title_w = to_wide(&title);

    let timed_out = Arc::new(AtomicBool::new(false));
    spawn_timeout_closer(title_w.clone(), timeout, timed_out.clone());

    let flags = btntype.to_u32() | msgtype.to_u32() | MB_SETFOREGROUND | MB_SYSTEMMODAL;
    let result = unsafe { MessageBoxExW(0, text_w.as_ptr(), title_w.as_ptr(), flags, 0) };

    if timed_out.load(Ordering::SeqCst) {
        -1
    } else {
        result
    }
}

/// Show an information message box
///
/// ### Parameters
/// - `msg`: Message text
/// - `title`: Dialog title; defaults to "Information"
/// - `timeout`: Auto-close timeout in seconds
///
/// ### Behavior
/// - Blue information icon
/// - OK button only
/// - For informational feedback
#[allow(dead_code)]
pub fn info_msgbox(msg: impl ToString, title: impl ToString, timeout: u32) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Info, MsgBtnType::Ok, timeout)
}

/// Show an error message box
///
/// ### Parameters
/// - `msg`: Error text
/// - `title`: Dialog title; defaults to "Error"
/// - `timeout`: Auto-close timeout in seconds
///
/// ### Behavior
/// - Red error icon
/// - OK button only
/// - For error/exception display
#[allow(dead_code)]
pub fn error_msgbox(msg: impl ToString, title: impl ToString, timeout: u32) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Error, MsgBtnType::Ok, timeout)
}

/// Show a warning message box
///
/// ### Parameters
/// - `msg`: Warning text
/// - `title`: Dialog title; defaults to "Warning"
/// - `timeout`: Auto-close timeout in seconds
///
/// ### Returns
/// - `i32`: Button result (usually OK)
///
/// ### Behavior
/// - Yellow warning icon
/// - OK button only
/// - For cautions and notices
#[allow(dead_code)]
pub fn warn_msgbox(msg: impl ToString, title: impl ToString, timeout: u32) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Warn, MsgBtnType::Ok, timeout)
}

/// Show a Yes/No question dialog
///
/// ### Parameters
/// - `msg`: Question text
/// - `title`: Dialog title; defaults to "Question"
/// - `timeout`: Auto-close timeout in seconds
///
/// ### Returns
/// - `i32`: Button code
///   - 6: Yes
///   - 7: No
///
/// ### Behavior
/// - Blue question icon
/// - Yes and No buttons
/// - For binary confirmations
#[allow(dead_code)]
pub fn quest_msgbox_yesno(msg: impl ToString, title: impl ToString, timeout: u32) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Quest, MsgBtnType::YesNo, timeout)
}

/// Show an OK/Cancel question dialog
///
/// ### Parameters
/// - `msg`: Question text
/// - `title`: Dialog title; defaults to "Question"
/// - `timeout`: Auto-close timeout in seconds
///
/// ### Returns
/// - `i32`: Button code
///   - 1: OK
///   - 2: Cancel
///
/// ### Behavior
/// - Blue question icon
/// - OK and Cancel buttons
/// - For operation confirmations
#[allow(dead_code)]
pub fn quest_msgbox_okcancel(msg: impl ToString, title: impl ToString, timeout: u32) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Quest, MsgBtnType::OkCancel, timeout)
}
