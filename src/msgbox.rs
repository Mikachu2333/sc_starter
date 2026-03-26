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
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

static PROCESS_NAME: &str = std::env!("CARGO_PKG_NAME");

/// Global list of notification window threads
static NOTIFY_THREADS: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

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
unsafe extern "system" {
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
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: isize,
        hInstance: isize,
        lpParam: *mut std::ffi::c_void,
    ) -> HWND;
    fn DestroyWindow(hWnd: HWND) -> i32;
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

const NIM_MODIFY: u32 = 0x00000001;
const NIF_INFO: u32 = 0x00000010;
const NIIF_INFO: u32 = 0x00000001;

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
unsafe extern "system" {
    fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
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

// ============================================================================
// Custom Notification Window Implementation
// ============================================================================

// Window styles
const WS_POPUP: u32 = 0x80000000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const WS_VSCROLL: u32 = 0x00200000;
const WS_EX_TOPMOST: u32 = 0x00000008;
const WS_EX_NOACTIVATE: u32 = 0x08000000;
const WS_EX_TOOLWINDOW: u32 = 0x00000080;

// Edit control styles
const ES_MULTILINE: u32 = 0x0004;
const ES_READONLY: u32 = 0x0800;
const ES_AUTOVSCROLL: u32 = 0x0040;

// Window messages
const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_TIMER: u32 = 0x0113;
const WM_SETFONT: u32 = 0x0030;
const WM_NCDESTROY: u32 = 0x0082;

// System parameters
const SPI_GETWORKAREA: u32 = 0x0030;

// GetWindowLongPtr index
const GWLP_USERDATA: i32 = -21;

// Timer ID for auto-close
const TIMER_ID_AUTOCLOSE: usize = 1;

// Windows notification typical size at 96 DPI
const NOTIFY_WIDTH_96DPI: i32 = 364;
const NOTIFY_HEIGHT_96DPI: i32 = 109;

// Font parameters
const FW_NORMAL: i32 = 400;
const DEFAULT_CHARSET: u32 = 1;
const OUT_DEFAULT_PRECIS: u32 = 0;
const CLIP_DEFAULT_PRECIS: u32 = 0;
const CLEARTYPE_QUALITY: u32 = 5;
const VARIABLE_PITCH: u32 = 2;
const FF_SWISS: u32 = 0x20;

// ShowWindow commands
const SW_SHOWNA: i32 = 8;

// DPI awareness context
const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
struct WNDCLASSEXW {
    cbSize: u32,
    style: u32,
    lpfnWndProc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> isize,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: isize,
    hIcon: isize,
    hCursor: isize,
    hbrBackground: isize,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: isize,
}

#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
struct MSG {
    hwnd: HWND,
    message: u32,
    wParam: WPARAM,
    lParam: LPARAM,
    time: u32,
    pt_x: i32,
    pt_y: i32,
}

/// Data passed to the notification window
struct NotifyWindowData {
    edit_hwnd: HWND,
    font: isize,
}

#[link(name = "User32")]
unsafe extern "system" {
    fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
    fn UnregisterClassW(lpClassName: *const u16, hInstance: isize) -> i32;
    fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostQuitMessage(nExitCode: i32);
    fn SetTimer(hWnd: HWND, nIDEvent: usize, uElapse: u32, lpTimerFunc: *const ()) -> usize;
    fn KillTimer(hWnd: HWND, uIDEvent: usize) -> i32;
    fn SystemParametersInfoW(
        uiAction: u32,
        uiParam: u32,
        pvParam: *mut std::ffi::c_void,
        fWinIni: u32,
    ) -> i32;
    fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
    fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> i32;
    fn MoveWindow(hWnd: HWND, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: i32) -> i32;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
    fn SendMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    fn GetDpiForWindow(hwnd: HWND) -> u32;
    fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> i32;
    fn SetThreadDpiAwarenessContext(dpiContext: isize) -> isize;
}

#[link(name = "Gdi32")]
unsafe extern "system" {
    fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: u32,
        bUnderline: u32,
        bStrikeOut: u32,
        iCharSet: u32,
        iOutPrecision: u32,
        iClipPrecision: u32,
        iQuality: u32,
        iPitchAndFamily: u32,
        pszFaceName: *const u16,
    ) -> isize;
    fn DeleteObject(ho: isize) -> i32;
}

/// Window procedure for the notification window
unsafe extern "system" fn notify_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    unsafe {
        match msg {
            WM_CREATE => {
                // Get DPI for proper scaling
                let dpi = GetDpiForWindow(hwnd);
                let dpi = if dpi == 0 { 96 } else { dpi };
                let scale = dpi as f32 / 96.0;

                // Create font scaled for DPI (Microsoft YaHei UI, 12pt at 96 DPI)
                let font_height = -(12.0 * scale) as i32;
                let font_name = to_wide("Microsoft YaHei UI");
                let font = CreateFontW(
                    font_height,
                    0,
                    0,
                    0,
                    FW_NORMAL,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET,
                    OUT_DEFAULT_PRECIS,
                    CLIP_DEFAULT_PRECIS,
                    CLEARTYPE_QUALITY,
                    VARIABLE_PITCH | FF_SWISS,
                    font_name.as_ptr(),
                );

                // Create the edit control for text display
                let edit_class = to_wide("EDIT");
                let edit_hwnd = CreateWindowExW(
                    0,
                    edit_class.as_ptr(),
                    ptr::null(),
                    WS_CHILD
                        | WS_VISIBLE
                        | WS_VSCROLL
                        | ES_MULTILINE
                        | ES_READONLY
                        | ES_AUTOVSCROLL,
                    0,
                    0,
                    0,
                    0,
                    hwnd,
                    0,
                    0,
                    ptr::null_mut(),
                );

                if edit_hwnd != 0 {
                    // Set the font
                    SendMessageW(edit_hwnd, WM_SETFONT, font as WPARAM, 1);

                    // Resize the edit control to fill the client area
                    let mut rect: RECT = std::mem::zeroed();
                    GetClientRect(hwnd, &mut rect);
                    MoveWindow(
                        edit_hwnd,
                        0,
                        0,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        1,
                    );
                }

                // Store data in window user data
                let data = Box::new(NotifyWindowData { edit_hwnd, font });
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as isize);

                0
            }
            WM_TIMER => {
                if wparam == TIMER_ID_AUTOCLOSE {
                    KillTimer(hwnd, TIMER_ID_AUTOCLOSE);
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            WM_NCDESTROY => {
                // Clean up allocated data
                let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NotifyWindowData;
                if !data_ptr.is_null() {
                    let data = Box::from_raw(data_ptr);
                    if data.font != 0 {
                        DeleteObject(data.font);
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Displays a standalone notification window in the bottom-right corner of the screen.
///
/// This function creates a custom popup window that appears above the taskbar,
/// displaying the specified title and message. The window automatically closes
/// after the specified timeout.
///
/// ### Parameters
/// - `title`: Window title text
/// - `msg`: Notification message text (displayed in a scrollable, read-only text box)
/// - `_icon_type`: Reserved for compatibility (currently unused)
/// - `timeout_ms`: Time in milliseconds before the window automatically closes
///
/// ### Returns
/// - `true` on success
/// - `false` on failure
///
/// ### Features
/// - Window appears in the bottom-right corner, above the taskbar
/// - Always on top but does not steal focus
/// - Automatically scales with system DPI
/// - Text box supports word wrap and vertical scrolling for long messages
/// - Only has a close button (no minimize/maximize)
#[allow(dead_code)]
pub fn notify_msgbox_standalone(
    title: impl ToString,
    msg: impl ToString,
    timeout_ms: u64,
) -> bool {
    let title_str = title.to_string();
    let msg_str = msg.to_string();

    let handle = thread::spawn(move || {
        unsafe {
            // Enable Per-Monitor DPI awareness for this thread
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

            // Generate a unique class name using timestamp
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let class_name_str = format!("NotifyWnd_{}", timestamp);
            let class_name = to_wide(&class_name_str);

            // Register window class
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: notify_wnd_proc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: 0,
                hIcon: 0,
                hCursor: 0,
                hbrBackground: 16, // COLOR_WINDOW + 1
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: 0,
            };

            if RegisterClassExW(&wc) == 0 {
                return;
            }

            // Get work area (screen area excluding taskbar)
            let mut work_area: RECT = std::mem::zeroed();
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                &mut work_area as *mut RECT as *mut std::ffi::c_void,
                0,
            );

            // Create a temporary window to get DPI using system class
            // to avoid triggering PostQuitMessage from our custom wndproc
            let static_class = to_wide("STATIC");
            let temp_hwnd = CreateWindowExW(
                0,
                static_class.as_ptr(),
                ptr::null(),
                0,
                0,
                0,
                1,
                1,
                0,
                0,
                0,
                ptr::null_mut(),
            );

            let dpi = if temp_hwnd != 0 {
                let d = GetDpiForWindow(temp_hwnd);
                DestroyWindow(temp_hwnd);
                if d == 0 { 96 } else { d }
            } else {
                96
            };

            // Scale dimensions for DPI
            let scale = dpi as f32 / 96.0;
            let width = (NOTIFY_WIDTH_96DPI as f32 * scale) as i32;
            let height = (NOTIFY_HEIGHT_96DPI as f32 * scale) as i32;

            // Position: bottom-right, above taskbar
            let x = work_area.right - width;
            let y = work_area.bottom - height;

            // Create the notification window
            let title_w = to_wide(&title_str);
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                title_w.as_ptr(),
                WS_POPUP | WS_CAPTION | WS_SYSMENU,
                x,
                y,
                width,
                height,
                0,
                0,
                0,
                ptr::null_mut(),
            );

            if hwnd == 0 {
                UnregisterClassW(class_name.as_ptr(), 0);
                return;
            }

            // Set the message text in the edit control
            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NotifyWindowData;
            if !data_ptr.is_null() {
                let msg_w = to_wide(&msg_str);
                SetWindowTextW((*data_ptr).edit_hwnd, msg_w.as_ptr());
            }

            // Set auto-close timer
            if timeout_ms > 0 {
                SetTimer(hwnd, TIMER_ID_AUTOCLOSE, timeout_ms as u32, ptr::null());
            }

            // Show window without activating
            ShowWindow(hwnd, SW_SHOWNA);

            // Message loop
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Cleanup
            UnregisterClassW(class_name.as_ptr(), 0);
        }
    });

    // Store the thread handle for later joining
    if let Ok(mut threads) = NOTIFY_THREADS.lock() {
        threads.push(handle);
    }

    true
}

/// Waits for all notification windows to close.
///
/// Call this before the main thread exits to ensure all notification
/// windows have been properly closed and cleaned up.
#[allow(dead_code)]
pub fn wait_notifications() {
    if let Ok(mut threads) = NOTIFY_THREADS.lock() {
        for handle in threads.drain(..) {
            let _ = handle.join();
        }
    }
}
