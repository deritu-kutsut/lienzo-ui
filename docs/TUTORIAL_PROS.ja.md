# プロ向けチュートリアル

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

## エクスポートで生成されるもの

すべて Unity プロジェクトの `Assets/EditorGUI` に書き出されます：

| ファイル | 用途 |
|---|---|
| `Designs/<Design>.json` | デザインデータ。変更されるたびにインポーターが Prefab を再構築します。 |
| `Editor/EditorGuiImporter.cs` | uGUI + TextMeshPro で Prefab を構築します。**Tools › Lienzo UI** メニュー。 |
| `Prefabs/<Design>.prefab` | すぐに使える Prefab。 |
| `UI Toolkit/<Design>.uxml` + `.uss` | UI Toolkit 版（UIDocument、PanelSettings 1920×1080 Scale With Screen Size）。 |
| `Scripts/<Design>OnGUI.cs` | IMGUI 版：GameObject に追加して使います。 |
| `Scripts/<Design>ViewBase.cs` | 型付き参照とイベント（再生成されます）。 |
| `Scripts/<Design>View.cs` | **あなた自身の**ロジック：初回のみ作成され、上書きされません。 |
| `Sprites/`, `Fonts/` | デザインで使用されている画像とフォント。 |

`<Design>` はアクセント記号を除いた PascalCase のタイトルです（`Menú principal` → `MenuPrincipal`）。

画像が見つからない場合は何も書き出されません。プロジェクトに変更を加える前にエクスポートが停止し、読み込めなかったファイルが通知されます。

## アンカーとスケーリング

- 何も選択していない状態では、インスペクターに **Canvas Scaler**（基準解像度、*match*、背景）が表示されます。
- 各要素は `RectTransform` と同様にアンカー、ピボット、オフセットを持ちます。基準解像度を変更すると、要素はアンカーに従って追従します。
- 内蔵テンプレートは 1920×1080 で設計されています。より小さいページや縦向きのページでは、画面に収まるよう均一に縮小されます。
- エクスポートする前に、**テスト** (F5) で複数の画面サイズを確認してください。

## レイアウトとリスト

コンテナは、子要素を水平・垂直・グリッド（Unity の *Layout Group* と同様）に余白（スペース）とパディング付きで整列できます。ScrollView はコンテンツのサイズを自動計算します。ヒエラルキーの順序がそのまま描画順になります。

## ロジック

- **エディター内**：テストモードでは、各要素のクリック時に Rhai スクリプト（`show("Name")`、`hide`、`toggle`、`set_text`、`set_color`、`move_by` など）を実行できます。画面遷移のプロトタイプ作成に便利です。テスト中に行われた変更はテストモード終了時に破棄され、保存やエクスポートはされません。
- **Unity 内**：`<Design>View` は `<Design>ViewBase` を継承し、ボタン・Slider・Toggle・Input Field ごとのフィールドと、オーバーライド用の `On<Element>Click` / `On<Element>Changed` メソッドを備えています。

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

操作可能な要素には一意の名前を付けてください。スクリプトや `GameObject.Find` は最初に一致した要素のみを検出します。**チェック** は重複した名前を警告します。

## デザインをチェック

**チェック** ボタン（およびコマンドラインの `--review`）は、基準解像度において以下の項目を検査します：

| チェック項目 | ルール |
|---|---|
| 文字サイズ | 1080p で 18 px 以上 |
| コントラスト | WCAG：4.5:1、または大きな文字（24 px、または太字 18.7 px）は 3:1。無効化されたコントロールは対象外 |
| タッチサイズ | ボタン、チェックボックス、入力欄、ドロップダウンの短辺が 1080p で 64 px 以上 |
| ページ | ページの完全または一部外にある要素 |
| 名前 | 操作可能な要素での名前の重複 |
| 画像 | 存在しなくなった画像ファイル |

2:1 未満のコントラスト、完全にページ外にある要素、見つからない画像はエラーとなり、それ以外は警告となります。

## コマンドライン

```bash
lienzo-ui --templates                                   # テンプレート一覧を表示 (0-12)
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # エラーがある場合は終了コード 1
lienzo-ui --open inventory.editorgui.json --export "C:/MyGame"   # ウィンドウなし
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # 指定の画面サイズでテストモードを開く
lienzo-ui --tutorial [--pro]                     # チュートリアルを強制表示
lienzo-ui --no-tutorial                                 # チュートリアルを開かない
```

`--save`、`--export`、`--review` はウィンドウを開きません。スクリプトや CI、[MCP サーバー](MCP.md)（`review_design` ツール）で使用します。

## ソースコード

Rust + [egui](https://github.com/emilk/egui)：

| ファイル | 内容 |
|---|---|
| `src/model.rs` | データ：`Doc`、`Element`、`Kind`、アンカー、レイアウト、テンプレート |
| `src/canvas.rs` | キャンバス：選択、ハンドル、ガイド、ズーム |
| `src/panels.rs` | 上部バー、サイドバー、インスペクター、ウィンドウ |
| `src/app.rs` | 状態、元に戻す、コマンド、ショートカット |
| `src/render.rs` | 要素とフォントの描画 |
| `src/revision.rs` | デザインをチェックの検査処理 |
| `src/export/` | エクスポーター：`ugui.rs`、`uitk.rs`、`imgui.rs`、`view.rs` および Unity インポーター |
| `src/tutorial.rs` | 内蔵チュートリアル（スペイン語・英語ページ） |
| `src/script.rs` | テストモード用の Rhai スクリプト |

拡張方法については [EXTENDING.md](EXTENDING.md) を参照してください。
