# sc_starter

A starter for ScreenCapture

## 功能说明 / Features

本软件是 [ScreenCapture](https://github.com/xland/ScreenCapture) 的启动器，提供以下功能：

- 内置截图程序，无需额外安装 / Related exe is embedded
- 自动注册全局快捷键 / Auto reg hotkey
- 支持自定义保存路径 / Support save pics in custom dir/path
- 文件防删除保护 / Avoid related file deletion
- 托盘左键单击截图，右键退出 / Left click tray icon to shot and Right to exit
- 自启动设置 / Support Auto Startup

## 使用方法 / Usage

1. 默认快捷键 / Default Hotkeys:
   - `Ctrl+Win+Alt+P`: 截屏 / Screen capture
   - `Ctrl+Win+Alt+L`: 长截屏 / Long screenshot
   - `Ctrl+Win+Alt+T`: 钉图 / Pin image
   - `Win+Ctrl+Shift+Esc`: 退出 / Exit
   - `Ctrl+Win+Alt+O`: 打开配置 / Open settings

2. 自定义设置 / Custom Settings:
   - 使用 `Ctrl+Win+Alt+O` 打开配置文件 / Use `Ctrl+Win+Alt+O` to open config file
   - 修改后保存并关闭文件 / Save and close file after modification
   - 使用 `Win+Ctrl+Shift+Esc` 退出程序 / Use `Win+Ctrl+Shift+Esc` to exit program
   - 重新启动程序应用新配置 / Restart program to apply new settings

## 配置说明 / Configuration

配置文件包含三个部分：

### [hotkey] 快捷键设置 / Hotkey Settings

- 格式：`控制键1+控制键2+...@实际键`
- 示例：`Ctrl+Win+Alt@P`
- 至少需要两个控制键
- 支持的控制键：Win/Ctrl/Alt/Shift
- 支持的实际键：字母A-Z、数字0-9、功能键F1-F24等

### [path] 保存路径设置 / Save Path Settings

- `&`: 每次手动选择位置（默认）
- `@`: 保存到桌面
- `*`: 保存到图片文件夹
- 自定义路径：如 `D:/Screenshots`
- 路径分隔符**必须使用** `/` 或 `\\`

### [sundry] 其他设置 / Other Settings

- `startup`: 是否自启动（0=否，1=是）
- `gui_config`: 普通截图工具栏配置
- `long_gui_config`: 长截图工具栏配置

## 注意事项 / Notes

1. 程序会自动以单例模式运行，防止多开
2. 配置文件自动保存在 `AppData/Local/SC_Starter` 目录
3. 程序会自动监控核心文件，防止被误删除
4. 所有快捷键不区分大小写
5. 修改配置后必须重启程序才能生效

Program will automatically run in singleton mode to prevent multiple instances
Configuration file is automatically saved in `AppData/Local/SC_Starter` directory  
Program will automatically monitor core files to prevent accidental deletion
All hotkeys are case-insensitive
Program restart is required after configuration changes

## 附，完整设置文件

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

[sundry]
# 设置是否开机自启
# Configure whether to start automatically at boot
# true->启用, false->禁用
startup = false

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
long_gui_config = "pin,clipboard,save,close"

```
