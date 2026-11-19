# 进阶教程

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROFESIONALES.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

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

想扩展它：[EXTENDER.md](EXTENDER.md)。
