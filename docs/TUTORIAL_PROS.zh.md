# 进阶教程

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

## 导出会生成什么

所有内容都写入 Unity 项目中的 `Assets/EditorGUI`：

| 文件 | 用途 |
|---|---|
| `Designs/<Design>.json` | 设计文件。只要它发生变化，导入器就会重新生成 prefab。 |
| `Editor/EditorGuiImporter.cs` | 用 uGUI + TextMeshPro 构建 prefab。**Tools › Lienzo UI** 菜单。 |
| `Prefabs/<Design>.prefab` | 可直接使用。 |
| `UI Toolkit/<Design>.uxml` + `.uss` | UI Toolkit 版本（UIDocument，PanelSettings 为 1920×1080 Scale With Screen Size）。 |
| `Scripts/<Design>OnGUI.cs` | IMGUI 版本：把它添加到某个 GameObject 上。 |
| `Scripts/<Design>ViewBase.cs` | 强类型引用和事件（会重新生成）。 |
| `Scripts/<Design>View.cs` | **你的**逻辑：只创建一次，永不覆盖。 |
| `Sprites/`、`Fonts/` | 设计里用到的图片和字体。 |

`<Design>` 是采用 PascalCase、去掉重音符号后的标题（`Menú principal` → `MenuPrincipal`）。

如果缺少某张图片，什么都不会写入：导出会在改动项目之前停下，并告诉你是哪个文件无法读取。

## 锚点与缩放

- 没有选中任何元素时，Inspector 会显示 **Canvas Scaler**：参考分辨率、*match* 和背景。
- 每个元素都有类似 `RectTransform` 的锚点、轴心和偏移。参考分辨率变化时，元素会跟随各自的锚点。
- 内置模板按 1920×1080 设计；在更小的页面或竖屏页面上，它们会等比缩小以适配。
- 导出前用**测试**（F5）在多种屏幕上检查一下。

## 布局与列表

容器可以横向、纵向或按网格排列子元素（类似 Unity 的 *Layout Groups*），并可设置间距和内边距。ScrollView 会计算自己的内容高度。层级的顺序就是绘制顺序。

## 逻辑

- **在编辑器里**：在测试模式下，每个元素可以在单击时运行一段 Rhai 脚本（`show("Name")`、`hide`、`toggle`、`set_text`、`set_color`、`move_by`…）。很适合用来做流程原型。测试时做的改动在退出测试模式后会被丢弃，既不会保存也不会导出。
- **在 Unity 里**：`<Design>View` 继承自 `<Design>ViewBase`，每个按钮、滑块、开关或输入框对应一个字段，并带有可重写的 `On<Element>Click` / `On<Element>Changed` 方法。

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

给交互元素起唯一的名称：脚本和 `GameObject.Find` 只会找到第一个同名对象。**检查**会提醒名称重复的问题。

## 检查设计

**检查**按钮（以及命令行上的 `--review`）会在参考分辨率下检查：

| 检查项 | 规则 |
|---|---|
| 文字大小 | 在 1080 p 下至少 18 px |
| 对比度 | WCAG：4.5:1，大号文字（24 px，或加粗的 18.7 px）为 3:1；禁用的控件不受此限 |
| 触摸尺寸 | 在 1080 p 下，按钮、开关、输入框和下拉菜单的短边至少 64 px |
| 页面 | 完全或部分超出页面的元素 |
| 名称 | 交互元素上重复的名称 |
| 图片 | 已不存在的图片文件 |

对比度低于 2:1、元素完全超出页面以及图片缺失属于错误；其余都是警告。

## 命令行

```bash
lienzo-ui --templates                                   # 列出模板（0-12）
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # 如果有错误，退出码为 1
lienzo-ui --open inventory.editorgui.json --export "C:/MyGame"   # 不打开窗口
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # 以该屏幕尺寸的测试模式打开
lienzo-ui --tutorial [--pro]                     # 强制显示教程
lienzo-ui --no-tutorial                                 # 不打开教程
```

`--save`、`--export` 和 `--review` 不会打开窗口：适合用于脚本、CI 和 [MCP 服务器](MCP.md)（`review_design` 工具）。

在 --export 后加上 --dry-run，会把每个文件列为 new、update、same 或 remove，而不写入任何内容。

export_unity 会直接写入 AI 所指定项目的 Assets/EditorGUI，编辑器中不会弹出任何提示：批准由你的 MCP 客户端的工具批准来完成。请先让 AI 用 dry_run: true 调用它，查看列表，然后再真正导出。

## 源代码

Rust + [egui](https://github.com/emilk/egui)：

| 文件 | 内容 |
|---|---|
| `src/model.rs` | 数据：`Doc`、`Element`、`Kind`、锚点、布局、模板 |
| `src/canvas.rs` | 画布：选中、手柄、参考线、缩放 |
| `src/panels.rs` | 顶栏、侧边栏、Inspector、窗口 |
| `src/app.rs` | 状态、撤销、命令、快捷键 |
| `src/render.rs` | 绘制元素和字体 |
| `src/revision.rs` | 检查设计的各项检查 |
| `src/export/` | 导出器：`ugui.rs`、`uitk.rs`、`imgui.rs`、`view.rs` 以及 Unity 导入器 |
| `src/tutorial.rs` | 内置教程（西班牙语和英语页面） |
| `src/script.rs` | 测试模式使用的 Rhai 脚本 |

想扩展它：[EXTENDING.md](EXTENDING.md)。

## 竖屏布局详解

竖屏布局是同一个设计的第二个视图（docs/FORMAT.md › PortraitLayout）：

- 它有自己的参考分辨率，即「竖屏尺寸」（默认 1080 × 1920）。
- 没有覆盖设置的元素保留其横屏 RectTransform，并按竖屏页面重新计算：全屏背景会拉伸，右侧锚定的元素跟随右边缘。
- 只有你在竖屏视图中修改的内容才会保存，形式是覆盖设置（位置、大小、旋转、锚点、可见性、字号、布局）。右键 ›「恢复为横屏」可移除其中一项。
- 导出：JSON 包含这些覆盖设置，生成的 LienzoOrientationLayout 脚本会在 Screen.width < Screen.height 时应用它们，尺寸变化时会再次应用。
- 「移除竖屏布局」会删除所有覆盖设置。

模板自带已做好的竖屏布局，以横屏布局的覆盖项形式保存，所以竖屏视图一开始就是为手机设计的版面。

## 9-slice 与内置素材包

适用于需要拉伸的边框、按钮和面板：

- 选中一个图片，在「图像设置 (Unity)」中设置「9-slice」：以原图像素为单位的边界（左、下、右、上），与 Unity 的 Sprite Editor 一致。导出时会生成 9-slice Sprite。
- 「每单位像素数」：100 让边界保持其像素大小，200 则缩小一半。
- 内置素材包（assets/pack/manifest.json、docs/CONTENT.md）已经设好切分和每单位像素数：边框为 512×512、切分 96，按钮为 512×192。
- 使用某一项会把它以 pack_<id>.<ext> 的名字复制到你的素材库（在「你的图片」中隐藏）；导出器会把它复制到 Unity 的 Sprites 中。
- 模板「像素风RPG界面」和「像素风主菜单」使用像素素材包。请开启「像素艺术」以使用 Point 过滤。

## 检查与快速修复

「检查」窗口会列出错误和警告；点击其中一项即可选中对应元素。有六类问题带有「修复」按钮（算一步撤销）：

- 文字小于最小字号：提高到最小值。
- 控件太小不便点击：围绕中心放大。
- 元素部分超出页面：如果放得下，就移到页面内。
- 名称重复：自动加上 _2、_3…
- 缺少图片：移除该引用。
- 对比度不足：加深或减淡文字颜色。

「修复所有可安全修复的问题」会分几轮全部处理。字体缺少字符之类的问题没有自动修复。这些修复在竖屏布局中同样有效。命令行：lienzo-ui --open 文件 --review（有错误时退出码为 1）。

## 从 Unity 导入：prefab 与命令行

文件 ›「从 Unity 导入…」可读取 uGUI 的 .prefab（Unity 的文本 YAML）或 Lienzo UI 导出的 Designs/*.json，并创建一个普通设计。

- 无法映射的内容会以警告列出。
- 检测到 Unity 项目文件夹时，本次会话中的图片会相对它来查找。
- 无窗口模式：lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json（也可配合 --review 和 --export）。
- 整个项目：lienzo-ui --import-unity-project MyProject --out Imported 会转换所有界面 prefab。它从不覆盖已有文件（会加上 _2），并跳过没有界面的 prefab。
- 详情见 docs/UNITY_IMPORT.md。

## 社区：插件、字体与规则

在「社区」标签页中，点击「安装」即可一键添加共享内容：插件、字体、图片、模板、元素和主题。

- 社区插件会放进名为 community-<名称>-<哈希> 的文件夹。它会先写入临时文件夹，所以被中断的安装不会留下半个插件。安装后会出现在「插件」标签页。
- 它的脚本同样是 Rhai 脚本：在沙箱中运行，不能访问文件和网络，并且对运算次数、时间和内存有限制（docs/PLUGINS.md）。
- 安装的字体会加入「文字」标签页，并随设计一起导出到 Unity。
- 下载内容会与公布的 SHA-256 指纹比对；不一致则不会安装。
- 要发布内容，需要登录（GitHub 或 Google）、选择公开名称并接受规则。所有内容都会先经过审核，18+ 内容必须标注。参见 docs/TERMS.md、docs/COMMUNITY_GUIDELINES.md 和 docs/PRIVACY.md。

安装前，「预览」会用示例文字显示字体，或显示插件的命令、导出器、文件和源代码（只读）；不会安装任何内容，也不会计入下载次数。「翻译」只会在你点击时把该描述发送给免费的 MyMemory 服务（docs/PRIVACY.md）。

## 自动保存、恢复与撤销

- 编辑时大约每分钟，完整设计（包括竖屏布局）会写入应用的恢复文件夹。测试模式下从不写入。最多保留 10 份恢复副本。
- 启动时，如果有比项目文件更新的恢复副本，会提示你：恢复或丢弃。正常保存之后就不再需要它了。
- 可以在「无障碍 › 自动保存」中关闭。
- 撤销历史限制为 200 步和约 64 MB。最早的步骤被丢弃时，状态栏会显示「历史记录已精简」。
- 每个插件命令以及检查窗口中的每次「修复」都只算一步撤销。
