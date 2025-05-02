# sc_starter

A starter for ScreenCapture

## 本软件是[ScreenCapture](https://github.com/xland/ScreenCapture)的启动器，自带原程序并自动注册快捷键，无显示界面，仅凭借快捷键进行操控

## 使用方法：见下方配置文件

如需自定义快捷键请先使用 `Ctrl+Win+Alt=O` 打开配置文件，更改并保存后使用 `Win+Ctrl+Shift=VK_ESCAPE` 退出软件，再次打开软件即可使用自定义配置。

```ini
encoding=utf-8

; 设置文件使用说明：
;
; ①设置内容不区分大小写，但必须按照指定格式书写，不可更改设置的格式，否则将无法设置成功
;    ⚠️设置完请保存并关闭文件，然后再次打开程序⚠️
;
; ②如相关配置未成功应用，将使用默认配置（例如快捷键与已有的冲突，将使用默认快捷键）


[hotkey]
; 快捷键格式如下：
; 「控制键1」+「控制键2」+「……」@「实际键」
; 【⚠️注意】：为了避免您设定的快捷键与当前系统中其他软件使用的快捷键冲突，请至少选定两个「控制键」，且尽量不要使用「Ctrl」+「Shift」=「X」样式的快捷键（因其过于常见）。
;
; 可用的控制键列表如下（大小写均可）：
; WIN / WINDOWS / SUPER （Win键等同类型控制键）
; CTRL / CONTROL （Ctrl键）
; ALT （Alt键）
; SHIFT （Shift键）
;
; 可用的实际键列表如下（尽量精确大小写）：
; A -> Z （字母键，不区分大小写）
; 0 -> 9 （数字键，非小键盘）
; VK_TAB （Tab键）
; VK_ESCAPE （Esc键）
; VK_INSERT （Insert键）
; VK_NUMPAD0 -> VK_NUMPAD9 （小键盘数字键）
; VK_F1 -> VK_F24 （Fn键系列）

; 控制截屏
screen_capture = Ctrl+Win+Alt@P
; 将剪贴板中的图像钉到屏幕
pin_to_screen = Ctrl+Win+Alt@T
; 退出软件
exit = Win+Ctrl+Shift@VK_ESCAPE
; 打开配置文件
open_conf = Ctrl+Win+Alt@O


[path]
; 设置图片的自动保存位置，可选以下几种：
; &         -> 截图时手动选定（默认）
; @         -> 桌面
; *         -> 图片文件夹
; D:/test   -> 其他指定文件夹（支持目录中含有中文及空格，路径必须存在）
dir = &
```
