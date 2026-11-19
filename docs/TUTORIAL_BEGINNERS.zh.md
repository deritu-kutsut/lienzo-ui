# 新手教程

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_BEGINNERS.es.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

用 Lienzo UI 制作你的第一个界面，并第一次导出到 Unity。无需编程。

## 1. 安装并打开

1. 从 GitHub 的 Releases 页面或 itch.io 下载 `LienzoUI-windows.zip`。
2. 解压后打开 `lienzo-ui.exe`。
3. 第一次打开时会显示**教程**（就是现在这段内容，位于应用内）。看完教程，或者点击「跳过并前往编辑器」后，编辑器就会打开。想再看一次教程：点击顶栏的**教程**按钮，或按 **F1**。

想把应用切换成中文：点击**无障碍**按钮 › **语言** › 中文。

![应用内的教程](images/tutorial.png)

## 2. 界面

| 区域 | 里面有什么 |
|---|---|
| 顶栏 | 文件、编辑、视图、Canvas、**测试**、设计名称、**无障碍**、**检查**、**主题**、**教程**、**保存**、**代码…**、**导出到 Unity** |
| 左侧栏 | **层级**（图层）、**元素**、**模板**、**文字**、**图片**、**插件**、**社区** |
| 中间 | 画布：你的游戏界面（默认为 1920 × 1080） |
| 右侧 | **Inspector**：所选元素的属性 |

![编辑器](images/editor.png)

## 3. 从模板开始

在**模板**里挑一个布局，共 12 种：主菜单、游戏 HUD、对话框窗口、设置、背包、暂停菜单、游戏结束、关卡选择、加载界面、角色对话、排行榜或角色选择。模板里的所有内容都能修改，按 Ctrl+Z 可撤销修改。想从零开始：选「空白」。

## 4. 编辑

- 把元素从**元素**拖到画布上。**现成组件**里有按钮、生命条和法力条、卡片、徽章和背包格子。
- **文字**里有标题样式和 HUD 部件：得分、计时器、金币、伤害数字、连击、等级标签和字幕。
- 单击选中；拖动移动；拖角调整大小；顶部的手柄用来旋转。
- 双击文字即可输入。
- 在 **Inspector** 中修改颜色、边框、阴影、字体和锚点。

| 快捷键 | 操作 |
|---|---|
| Ctrl+Z / Ctrl+Y | 撤销 / 重做 |
| Ctrl+D / Del | 复制 / 删除 |
| Ctrl+G / Ctrl+Shift+G | 编组 / 取消编组 |
| 方向键（Shift） | 移动 1 px（10 px） |
| Ctrl+S | 保存 |
| F5 | 测试 |
| Ctrl+E | 导出到 Unity |
| F1 | 教程 |

## 5. 测试

点击**测试**（F5）：界面变成可交互的，你还可以切换到其他屏幕（手机、4:3、超宽屏）看它如何自适应。按 Esc 退出测试模式，设计会恢复到测试前的状态。

## 6. 检查

点击顶栏的**检查**。它会提醒文字太小看不清、对比度太低、按钮太小不好点击、元素超出页面、名称重复以及图片缺失。点击某条提示即可选中对应元素并修改它。

## 7. 导出到 Unity

1. 点击**导出到 Unity**（Ctrl+E）。第一次导出时，选择你的项目文件夹（包含 `Assets` 的那个）。
2. 回到 Unity：设计会以 prefab 的形式出现在 `Assets/EditorGUI/Prefabs` 中。
3. 把它拖进场景，或者使用菜单 **Tools › Lienzo UI › Import design into scene…**。

以后每次重新导出，Unity 都会自动更新这个 prefab。

## 接下来

- [进阶教程](TUTORIAL_PROS.zh.md)：导出会生成什么，以及把你的逻辑放在哪里。
- [MCP 服务器](MCP.md)：用日常语言让 AI 帮你修改。
