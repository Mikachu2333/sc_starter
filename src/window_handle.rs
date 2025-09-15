//! Windows 窗口和进程处理模块
//!
//! 本模块提供了以下功能：
//! - 窗口置顶操作
//! - 进程检测和查找
//! - Windows API 调用封装

// Windows API 声明
type HWND = *mut std::ffi::c_void;
type DWORD = u32;
type BOOL = i32;
type HANDLE = *mut std::ffi::c_void;

const HWND_TOPMOST: HWND = -1isize as HWND;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
const MAX_PATH: usize = 260;

#[repr(C)]
struct PROCESSENTRY32W {
    dw_size: DWORD,
    cnt_usage: DWORD,
    th32_process_id: DWORD,
    th32_default_heap_id: usize,
    th32_module_id: DWORD,
    cnt_threads: DWORD,
    th32_parent_process_id: DWORD,
    pc_pri_class_base: i32,
    dw_flags: DWORD,
    sz_exe_file: [u16; MAX_PATH],
}

#[link(name = "user32")]
extern "system" {
    fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: u32,
    ) -> BOOL;
    fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
    fn EnumWindows(
        lpEnumFunc: unsafe extern "system" fn(HWND, isize) -> BOOL,
        lParam: isize,
    ) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(dw_flags: DWORD, th32_process_id: DWORD) -> HANDLE;
    fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
}

/// 根据进程ID查找并置顶窗口
///
/// ### 参数
/// - `process_id`: 目标进程ID
///
/// ### 功能
/// - 枚举所有顶级窗口
/// - 找到属于指定进程的可见窗口
/// - 将窗口设置为置顶状态
pub unsafe fn set_window_topmost_by_pid(process_id: u32) {
    unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: isize) -> BOOL {
        let target_pid = lparam as u32;
        let mut window_pid: DWORD = 0;

        GetWindowThreadProcessId(hwnd, &mut window_pid);

        if window_pid == target_pid && IsWindowVisible(hwnd) != 0 {
            SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }

        1 // 继续枚举
    }

    EnumWindows(enum_window_proc, process_id as isize);
}

/// 检测指定程序是否正在运行
///
/// ### 参数
/// - `process_name`: 进程名称（如"notepad.exe"、"chrome.exe"）
///
/// ### 返回值
/// - `bool`: 如果进程正在运行返回true，否则返回false
///
/// ### 功能
/// - 创建系统进程快照
/// - 遍历所有运行的进程
/// - 比较进程名称（不区分大小写）
/// - 自动处理资源清理
pub unsafe fn is_process_running(process_name: impl ToString) -> bool {
    // 创建进程快照
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if snapshot == INVALID_HANDLE_VALUE {
        return false;
    }

    // 初始化进程条目结构
    let mut pe32 = PROCESSENTRY32W {
        dw_size: std::mem::size_of::<PROCESSENTRY32W>() as DWORD,
        cnt_usage: 0,
        th32_process_id: 0,
        th32_default_heap_id: 0,
        th32_module_id: 0,
        cnt_threads: 0,
        th32_parent_process_id: 0,
        pc_pri_class_base: 0,
        dw_flags: 0,
        sz_exe_file: [0; MAX_PATH],
    };

    let mut found = false;

    // 获取第一个进程
    if Process32FirstW(snapshot, &mut pe32) != 0 {
        loop {
            // 将进程名转换为String并转换为小写
            let exe_name = String::from_utf16_lossy(&pe32.sz_exe_file)
                .trim_end_matches('\0')
                .to_lowercase();

            // 比较进程名
            if exe_name == process_name.to_string().to_lowercase() {
                found = true;
                break;
            }

            // 获取下一个进程
            if Process32NextW(snapshot, &mut pe32) == 0 {
                break;
            }
        }
    }

    // 清理资源
    CloseHandle(snapshot);
    found
}