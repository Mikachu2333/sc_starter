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
    fn CreateWindowExW(
        dwExStyle: u32, lpClassName: *const u16, lpWindowName: *const u16,
        dwStyle: u32, x: i32, y: i32, nWidth: i32, nHeight: i32,
        hWndParent: HWND, hMenu: isize, hInstance: isize, lpParam: *mut std::ffi::c_void,
    ) -> HWND;
    fn DestroyWindow(hWnd: HWND) -> i32;
}

const HWND_MESSAGE: HWND = -3; // Message-only window parent

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

const NIM_ADD: u32 = 0x00000000;
const NIM_MODIFY: u32 = 0x00000001;
const NIM_DELETE: u32 = 0x00000002;
const NIF_ICON: u32 = 0x00000002;
const NIF_TIP: u32 = 0x00000004;
const NIF_INFO: u32 = 0x00000010;
const NIIF_INFO: u32 = 0x00000001;
const NIIF_WARNING: u32 = 0x00000002;
const NIIF_ERROR: u32 = 0x00000003;

/// Notification icon types for balloon tips
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum NotifyIconType {
    Info,
    Warning,
    Error,
}

#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms, dead_code)]
struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: UINT,
    pub uFlags: UINT,
    pub uCallbackMessage: UINT,
    pub hIcon: isize,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub uTimeoutOrVersion: UINT,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: [u8; 16],
    pub hBalloonIcon: isize,
}

#[link(name = "Shell32")]
extern "system" {
    fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
}

#[link(name = "User32")]
extern "system" {
    fn LoadIconW(hInstance: isize, lpIconName: *const u16) -> isize;
}

/// Displays a balloon tip notification on an existing system tray icon.
///
/// This function uses `Shell_NotifyIconW` with `NIM_MODIFY` to show a balloon
/// notification on a tray icon that has already been added to the system tray.
///
/// ### Parameters
/// - `hwnd`: Handle to the window that owns the tray icon
/// - `msg`: Notification message text (max 255 characters, will be truncated if longer)
/// - `icon_id`: The unique identifier (`uID`) of the existing tray icon to display the balloon on
///
/// ### Returns
/// - Non-zero value on success
/// - `0` on failure (e.g., if the tray icon with the specified `icon_id` does not exist)
///
/// ### Prerequisites
/// The tray icon identified by `icon_id` must have been previously added using
/// `Shell_NotifyIconW` with `NIM_ADD`. If the icon does not exist, this function will fail.
///
/// ### Example
/// ```ignore
/// // Assuming a tray icon with ID 1 has been added to the system tray
/// let result = notify_msgbox(hwnd, "Operation completed successfully", 1);
/// if result == 0 {
///     eprintln!("Failed to show notification");
/// }
/// ```
#[allow(dead_code)]
pub fn notify_msgbox(hwnd: HWND, msg: impl ToString, icon_id: u32) -> i32 {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = icon_id;
    nid.uFlags = NIF_INFO;
    nid.dwInfoFlags = NIIF_INFO;

    let title_w = to_wide(PROCESS_NAME);
    let msg_w = to_wide(msg);

    for (i, &c) in title_w.iter().take(63).enumerate() {
        nid.szInfoTitle[i] = c;
    }
    for (i, &c) in msg_w.iter().take(255).enumerate() {
        nid.szInfo[i] = c;
    }

    unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid) }
}

/// Unique icon ID for standalone notifications
const STANDALONE_NOTIFY_ICON_ID: u32 = 0xCCAA_7788;

/// IDI_APPLICATION = MAKEINTRESOURCE(32512)
const IDI_APPLICATION: *const u16 = 32512 as *const u16;

/// Displays a standalone balloon notification without requiring an existing tray icon.
///
/// This function creates a temporary system tray icon, shows a balloon notification,
/// and automatically removes the icon after a delay. Works on Windows 7 and above
/// without any prerequisites.
///
/// ### Parameters
/// - `title`: Notification title (max 63 characters, will be truncated if longer)
/// - `msg`: Notification message text (max 255 characters, will be truncated if longer)
/// - `icon_type`: Type of notification icon (`Info`, `Warning`, or `Error`)
/// - `timeout_ms`: Time in milliseconds before the tray icon is automatically removed
///   (the balloon itself follows system settings, typically 5-30 seconds)
///
/// ### Returns
/// - `true` on success
/// - `false` on failure
///
/// ### Example
/// ```ignore
/// // Show an info notification
/// show_notification("Download Complete", "Your file has been saved.", NotifyIconType::Info, 5000);
///
/// // Show a warning notification
/// show_notification("Low Disk Space", "Less than 1GB remaining.", NotifyIconType::Warning, 5000);
/// ```
///
/// ### Notes
/// - This function spawns a background thread to remove the tray icon after `timeout_ms`
/// - Multiple rapid calls may overlap; each call uses the same icon ID
/// - Compatible with Windows 7, 8, 8.1, 10, and 11
#[allow(dead_code)]
pub fn show_notification(
    title: impl ToString,
    msg: impl ToString,
    icon_type: NotifyIconType,
    timeout_ms: u64,
) -> bool {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = 0; // No window association needed
    nid.uID = STANDALONE_NOTIFY_ICON_ID;
    nid.uFlags = NIF_ICON | NIF_TIP | NIF_INFO;
    nid.hIcon = unsafe { LoadIconW(0, IDI_APPLICATION) };
    nid.dwInfoFlags = match icon_type {
        NotifyIconType::Info => NIIF_INFO,
        NotifyIconType::Warning => NIIF_WARNING,
        NotifyIconType::Error => NIIF_ERROR,
    };

    let title_w = to_wide(title.to_string());
    let msg_w = to_wide(msg.to_string());
    let tip_w = to_wide(PROCESS_NAME);

    // Set tooltip (shown on hover)
    for (i, &c) in tip_w.iter().take(127).enumerate() {
        nid.szTip[i] = c;
    }
    // Set balloon title
    for (i, &c) in title_w.iter().take(63).enumerate() {
        nid.szInfoTitle[i] = c;
    }
    // Set balloon message
    for (i, &c) in msg_w.iter().take(255).enumerate() {
        nid.szInfo[i] = c;
    }

    // First, try to delete any existing icon with the same ID (cleanup from previous calls)
    unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) };

    // 创建一个仅用于接收消息的隐藏系统窗口 (Message-only window)
    let class_name = to_wide("STATIC"); // 使用系统自带静态类
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            ptr::null(), // 无名
            0, 0, 0, 0, 0,
            HWND_MESSAGE, 
            0, 0, ptr::null_mut()
        )
    };

    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = STANDALONE_NOTIFY_ICON_ID;
    nid.uFlags = NIF_ICON | NIF_TIP | NIF_INFO;
    nid.hIcon = unsafe { LoadIconW(0, IDI_APPLICATION) };
    nid.dwInfoFlags = match icon_type {
        NotifyIconType::Info => NIIF_INFO,
        NotifyIconType::Warning => NIIF_WARNING,
        NotifyIconType::Error => NIIF_ERROR,
    };

    let title_w = to_wide(title);
    let msg_w = to_wide(msg);
    let tip_w = to_wide(PROCESS_NAME);

    // Set tooltip (shown on hover)
    for (i, &c) in tip_w.iter().take(127).enumerate() {
        nid.szTip[i] = c;
    }
    // Set balloon title
    for (i, &c) in title_w.iter().take(63).enumerate() {
        nid.szInfoTitle[i] = c;
    }
    // Set balloon message
    for (i, &c) in msg_w.iter().take(255).enumerate() {
        nid.szInfo[i] = c;
    }

    // First, try to delete any existing icon with the same ID (cleanup from previous calls)
    unsafe { Shell_NotifyIconW(NIM_DELETE, &nid) };

    // Add the tray icon and show balloon
    let result = unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };
    if result == 0 {
        return false;
    }

    // Spawn a thread to remove the icon after timeout
    thread::spawn(move || {
        thread::sleep(std::time::Duration::from_millis(timeout_ms));
        let mut cleanup_nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
        cleanup_nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        cleanup_nid.hWnd = hwnd; // 对应当初的hwnd
        cleanup_nid.uID = STANDALONE_NOTIFY_ICON_ID;
        unsafe { 
            Shell_NotifyIconW(NIM_DELETE, &cleanup_nid);
            DestroyWindow(hwnd); // 释放伪窗口
        };
    });

    true
}
