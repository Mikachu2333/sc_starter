# SC_Starter

本软件是 [ScreenCapture](https://github.com/xland/ScreenCapture) 的启动器

A launcher and manager for ScreenCapture with global hotkeys and tray integration.

## 功能说明 / Features

### 核心功能 / Core Features

- **内置截图程序** / Embedded ScreenCapture exe - 无需额外安装，一键运行
- **全局快捷键** / Global Hotkeys - 系统级快捷键，任意位置触发
- **启动应用程序** / Launch Applications - 快速启动常用程序，支持进程管理和窗口置顶
- **路径管理** / Path Management - 支持多种保存方式和自定义路径
- **文件保护机制** / File Protection - 自动监控和恢复核心文件
- **单例运行** / Singleton Mode - 防止程序多开造成冲突
- **托盘集成** / System Tray Integration - 便捷的托盘操作界面

### 托盘操作 / Tray Operations

- **左键双击** / Left Double Click: 普通截图 / Normal screenshot
- **右键单击** / Right Double Click: 长截图 / Long screenshot  
- **中键单击** / Middle Click: 退出程序 / Exit application

### 自启动支持 / Auto Startup

- **开机自启** / Boot Startup - 可配置的开机自动启动
- **配置管理** / Configuration Management - 自动创建和管理启动项

## 使用方法 / Usage Guide

### 1. 快捷键操作 / Hotkey Operations

#### 默认快捷键 / Default Hotkeys

| 功能 / Function          | 快捷键 / Hotkey      | 说明 / Description                                   |
| ------------------------ | -------------------- | ---------------------------------------------------- |
| 普通截图 / Screenshot    | `Ctrl+Win+Alt+P`     | 矩形区域截图 / Rectangular area capture              |
| 长截图 / Long Screenshot | `Ctrl+Win+Alt+L`     | 滚动截图 / Scrolling capture                         |
| 钉图 / Pin Image         | `Ctrl+Win+Alt+T`     | 将剪贴板图片钉到屏幕 / Pin clipboard image to screen |
| 启动应用 / Launch App    | `Ctrl+Win+Alt+A`     | 启动配置的外部应用程序 / Launch configured application |
| 打开配置 / Open Config   | `Ctrl+Win+Alt+O`     | 打开配置文件编辑 / Open config file for editing      |
| 退出程序 / Exit          | `Win+Ctrl+Shift+Esc` | 完全退出程序 / Exit application completely           |

### 2. 配置自定义 / Configuration Customization

#### 修改配置步骤 / Configuration Steps

1. 按 `Ctrl+Win+Alt+O` 打开配置文件 / Press `Ctrl+Win+Alt+O` to open config file
2. 根据需要修改设置 / Modify settings as needed
3. 保存并关闭配置文件 / Save and close the config file
4. 按 `Win+Ctrl+Shift+Esc` 退出程序 / Press `Win+Ctrl+Shift+Esc` to exit
5. 重新启动程序加载新配置 / Restart program to load new settings

## 配置详解 / Configuration Details

配置文件位于：`%LOCALAPPDATA%\SC_Starter\config.toml`

### [hotkey] 快捷键配置 / Hotkey Configuration

#### 格式说明 / Format Description

```toml
功能名 = "修饰键1+修饰键2+...@目标键"
function = "Modifier1+Modifier2+...@TargetKey"
```

#### 支持的修饰键 / Supported Modifiers

- `Win` / `Windows` / `Super`: Windows键
- `Ctrl` / `Control`: Ctrl键  
- `Alt`: Alt键
- `Shift`: Shift键

#### 支持的目标键 / Supported Target Keys

- **字母键** / Letters: `A-Z`
- **数字键** / Numbers: `0-9`
- **功能键** / Function Keys: `F1-F24`
- **特殊键** / Special Keys: `VK_ESCAPE`, `VK_SPACE`, `VK_TAB` 等

#### 配置要求 / Requirements

- **至少两个修饰键** / Minimum 2 modifiers required
- **避免系统快捷键冲突** / Avoid system hotkey conflicts

### [path] 路径配置 / Path Configuration

#### 截图保存路径 / Screenshot Save Path

| 设置值 / Value   | 功能 / Function              | 说明 / Description       |
| ---------------- | ---------------------------- | ------------------------ |
| `&`              | 手动选择 / Manual Selection  | 每次截图时弹出保存对话框 |
| `@`              | 桌面 / Desktop               | 自动保存到桌面           |
| `*`              | 图片文件夹 / Pictures Folder | 自动保存到用户图片文件夹 |
| `D:/Screenshots` | 自定义路径 / Custom Path     | 保存到指定文件夹         |

#### 启动应用程序配置 / Launch Application Configuration

**应用程序路径 / Application Path:**

- 支持可执行文件：`.exe`, `.bat`, `.cmd`, `.com`, `.msi`
- 支持文档文件：`.html`, `.txt`, `.pdf`, `.docx` 等（通过默认程序打开）
- 可执行文件支持进程管理和窗口置顶功能
- 文档文件每次都会重新打开

**命令行参数 / Command Line Arguments:**

- 多个参数用制表符（Tab）分隔
- 支持路径参数和开关参数
- 示例：`-fullscreen	--config=custom.ini`

**行为说明 / Behavior Description:**

- **可执行文件**：首次启动记录进程ID，再次按快捷键时检测进程状态
  - 如果进程运行中 → 窗口置顶显示
  - 如果进程已退出 → 启动新进程
- **文档文件**：每次都通过系统默认程序打开，不进行进程管理

#### 路径格式要求 / Path Format Requirements

- 使用正斜杠: `D:/Screenshots`  
- 使用双反斜杠: `D:\\Screenshots`
- 不要使用单反斜杠: `D:\Screenshots`

### [sundry] 杂项 / Advanced Settings

- **auto_start**: 开机自启动 (`true`/`false`)
- **comp_level**: 压缩级别 (0-100)
- **scale_level**: 缩放级别 (0-100)

### GUI 工具栏配置 / GUI Toolbar Configuration

**可用工具 / Available Tools:**

- `rect`: 矩形工具 / Rectangle tool
- `ellipse`: 椭圆工具 / Ellipse tool  
- `arrow`: 箭头工具 / Arrow tool
- `number`: 序号工具 / Number tool
- `line`: 直线工具 / Line tool
- `text`: 文本工具 / Text tool
- `mosaic`: 马赛克工具 / Mosaic tool
- `eraser`: 橡皮擦工具 / Eraser tool
- `undo`/`redo`: 撤销/重做 / Undo/Redo
- `pin`: 钉图功能 / Pin function
- `clipboard`: 复制到剪贴板 / Copy to clipboard
- `save`: 保存功能 / Save function
- `close`: 关闭功能 / Close function
- `|`: 分隔符 / Separator

## 完整配置文件示例 / Complete Configuration Example

```toml
# 设置文件使用说明 / Configuration File Instructions:
#
# ①程序启动时会自动处理配置文件
# ①The program will automatically process the configuration file when starting
#   - 如果文件不存在，会创建默认配置
#   - If the file does not exist, default settings will be created
#   - 如果某项配置缺失，会自动补充默认值
#   - If any setting is missing, default values will be added
#
# ②配置更改后需要重启程序才能生效
# ②Program restart is required after configuration changes
#   - 使用快捷键 Win+Ctrl+Shift+Esc 退出程序
#   - Use Win+Ctrl+Shift+Esc to exit the program
#   - 重新启动程序加载新配置
#   - Restart the program to load new settings

[hotkey]
# 快捷键配置说明 / Hotkey Configuration:
# 1. 格式：控制键+控制键+...@实际键
# 1. Format: Modifier+Modifier+...@Key
# 2. 至少需要两个控制键，避免冲突
# 2. At least two modifiers required to avoid conflicts
# 3. 支持的控制键 / Supported modifiers:
#    - WIN/WINDOWS/SUPER
#    - CTRL/CONTROL
#    - ALT
#    - SHIFT
# 4. 支持的实际键 / Supported keys:
#    - A-Z：字母键 / Letters
#    - 0-9：数字键 / Numbers
#    - VK_系列：特殊键 / Special keys

# 控制截屏 / Screen capture
screen_capture = "Ctrl+Win+Alt@P"

# 控制截长屏 / Screen capture long
screen_capture_long = "Ctrl+Win+Alt@L"

# 将剪贴板中的图像钉到屏幕 / Pin clipboard image to screen
pin_to_screen = "Ctrl+Win+Alt@T"

# 退出软件 / Exit application
exit = "Win+Ctrl+Shift@VK_ESCAPE"

# 打开配置文件 / Open configuration file
open_conf = "Ctrl+Win+Alt@O"

# 调用其它软件 / Launch other application
# 该软件将自动置顶 / The application will be set to topmost
# 软件路径在下方的 path 配置类别中指定
launch_app = "Ctrl+Win+Alt@A"

[path]
# 设置图片的自动保存位置，可选以下几种：
# Configure automatic save location for images, options:
# &         -> 截图时手动选定（默认）/ Manual selection when capturing (default)
# @         -> 桌面 / Desktop
# *         -> 图片文件夹 / Pictures folder
# D:/test   -> 其他指定文件夹（支持目录中含有中文及空格，路径必须存在）
#              Other specified folder (supports Chinese and spaces in path, must exist)
# ⚠️警告/Warning⚠️
# 路径必须使用斜杠『/』或双反斜杠『\\』
# Path must use slashes "/" or double backslashes "\\"
dir = "&"

# 需要启动的程序路径，要求同上
# Process path you want to launch
launch_app_path = "C:/Windows/System32/notepad.exe"
# 启动参数，可以置空
# 使用 Tab（打不出来可以复制这个<	>）间隔每个参数
# Process Args, blank is allowed
# Use Tab to separate each argument (it may not be typed, you can copy this <	>)
launch_app_args = ""

[sundry]
# 设置是否开机自启
# Configure whether to start automatically at boot
# true->启用, false->禁用
startup = false

# 图像压缩与缩放比例
# Image compression and scaling ratio settings
# 压缩等级：0-10（清晰->模糊），-1代表默认
# 缩放：1%-100%，（模糊->清晰）
# Compression level: 0-10 (clear->blur), -1 for default
# Scale: 1%-100% (blur->clear)
comp_level = -1
scale_ratio = 100

[gui]
# GUI配置，默认全部启用
# rect：方框
# ellipse：椭圆
# arrow：箭头
# number：标号
# line：直线
# text：文本
# mosaic：马赛克
# eraser：橡皮擦
# undo/redo：撤销/重做
# pin：钉图
# clipboard：保存到剪贴板
# save：保存
# close：关闭
gui_config = "rect,ellipse,arrow,number,line,text,mosaic,eraser,|,undo,redo,|,pin,clipboard,save,close"
# 参数只少不多
# Only fewer parameters are allowed
long_gui_config = "pin,clipboard,save,close"
```
