# sc_starter

A starter for ScreenCapture

## 功能说明 / Features

本软件是 [ScreenCapture](https://github.com/xland/ScreenCapture) 的启动器，提供以下功能：

- 内置截图程序，无需额外安装
- 自动注册全局快捷键
- 支持自定义保存路径
- 支持以时间戳命名
- 文件防删除保护
- 无界面设计，纯快捷键操作

## 使用方法 / Usage

1. 默认快捷键 / Default Hotkeys:
   - `Ctrl+Win+Alt+P`: 截屏 / Screen capture
   - `Ctrl+Win+Alt+T`: 钉图 / Pin image
   - `Win+Ctrl+Shift+Esc`: 退出 / Exit
   - `Ctrl+Win+Alt+O`: 打开配置 / Open settings

2. 自定义设置 / Custom Settings:
   - 使用 `Ctrl+Win+Alt+O` 打开配置文件
   - 修改后保存并关闭文件
   - 使用 `Win+Ctrl+Shift+Esc` 退出程序
   - 重新启动程序应用新配置

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

- `time`: 是否使用时间戳命名（0=否，1=是）
- 启用时间戳命名时必须指定保存路径

## 注意事项 / Notes

1. 程序会自动以单例模式运行，防止多开
2. 配置文件自动保存在 AppData/Local/SC_Starter 目录
3. 程序会自动监控核心文件，防止被误删除
4. 所有快捷键不区分大小写
5. 修改配置后必须重启程序才能生效
