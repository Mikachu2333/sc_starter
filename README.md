# sc_starter
A starter for ScreenCapture

## 本软件是[ScreenCapture](https://github.com/xland/ScreenCapture)的启动器，自带原程序并自动注册快捷键，无显示界面，仅凭借快捷键进行操控

# 使用方法：见下方配置文件

如需自定义快捷键请先使用`Ctrl+Alt=O`打开配置文件，更改并保存后使用`Win+Ctrl+Shift=VK_ESCAPE`退出软件，再次打开软件即可使用自定义配置。

```
Win+Alt=V
Win+Alt=C
Win+Ctrl+Shift=VK_ESCAPE
Ctrl+Alt=O

# 设置文件使用说明：
#
# ①设置内容不区分大小写，必须按照指定格式书写，否则将无法设置成功
#
#
# ②设置文件共4行，不可更改格式，不可删除其中任意一行
# 第一行：控制截屏
# 第二行：从剪贴板中的图像钉到屏幕
# 第三行：退出软件
# 第四行：打开配置文件
#
#
# ③格式如下：
# 「控制键1」+「控制键2」+「……」=「实际键」
# 【⚠️注意】：为了避免您设定的快捷键与当前系统中其他软件使用的快捷键冲突，请至少选定两个「控制键」，且尽量不要使用「Ctrl」+「Shift」=「X」样式的快捷键。如快捷键未成功设置，将使用默认快捷键。
#
# 可用的控制键列表如下（大小写均可）：
# WIN / WINDOWS / SUPER
# CTRL / CONTROL
# ALT
# SHIFT
#
# 可用的实际键列表如下：
# A -> Z
# a -> z
# 0 -> 9
# VK_TAB
# VK_ESCAPE
# VK_INSERT
# VK_NUMPAD0 -> VK_NUMPAD9
# VK_F1 -> VK_F24

```

此外，还想写个限制多开的来着，失败了（
