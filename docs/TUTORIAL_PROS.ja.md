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

--export に --dry-run を付けると、何も書き込まずに各ファイルを new、update、same、remove として一覧表示します。

export_unity は、AI が指定したプロジェクトの Assets/EditorGUI に直接書き込みます。エディターには確認が表示されず、承認は MCP クライアントのツール承認に依存します。AI にまず dry_run: true で呼び出してもらい、一覧を確認してから本番のエクスポートを行ってください。

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

## 縦向きレイアウトの詳細

縦向きレイアウトは、同じデザインの2つめのビューです（docs/FORMAT.md › PortraitLayout）。

- 独自の基準解像度「縦向きサイズ」を持ちます（既定は 1080 × 1920）。
- 上書きのない要素は横向きの RectTransform を保ち、縦向きのページを基準に再計算されます。全画面の背景は伸び、右にアンカーされた要素は右端に追従します。
- 保存されるのは縦向きビューで変更した内容だけで、上書きとして記録されます（位置、サイズ、回転、アンカー、可視性、フォントサイズ、レイアウト）。右クリック ›「横向きに戻す」で1つの上書きを取り除けます。
- エクスポート：JSON に上書きが含まれ、生成される LienzoOrientationLayout スクリプトが Screen.width < Screen.height のときに適用し、サイズが変わったときにも再適用します。
- 「縦向きレイアウトを削除」はすべての上書きを消します。

テンプレートには縦向きレイアウトが組み込み済みで、横向きレイアウトの上書きとして保存されているため、縦向きビューは最初からスマホ向けのレイアウトになります。

## 9-slice と内蔵パック

伸び縮みするフレーム、ボタン、パネル向けの機能です。

- 画像を選び、「画像設定 (Unity)」で「9-slice」を設定します。元画像のピクセル単位の境界（左、下、右、上）で、Unity の Sprite Editor と同じです。エクスポートすると 9-slice の Sprite が生成されます。
- 「ユニットあたりのピクセル数」：100 なら境界はピクセルサイズのまま、200 なら半分になります。
- 内蔵パック（assets/pack/manifest.json、docs/CONTENT.md）には、すでに分割位置とユニットあたりのピクセル数が設定されています。フレームは 512×512 で分割 96、ボタンは 512×192 です。
- 項目を使うと pack_<id>.<ext> という名前でライブラリへコピーされ（「マイ画像」には表示されません）、エクスポート時に Unity の Sprites へコピーされます。
- テンプレート「ピクセルRPG HUD」と「ピクセルメインメニュー」はピクセルパックを使っています。Point フィルターにするには「ピクセルアート」をオンにしてください。

## チェックとクイック修正

「チェック」ウィンドウにはエラーと警告が一覧表示され、クリックすると該当の要素が選択されます。6種類の問題には「修正」ボタンがあります（元に戻すのは1ステップ）。

- 最小サイズ未満の文字：最小サイズまで引き上げます。
- タップしにくい小さなコントロール：中心を保って拡大します。
- ページから一部はみ出した要素：収まる場合は内側へ移動します。
- 名前の重複：_2、_3… を付け足します。
- 欠落した画像：参照を削除します。
- コントラスト不足：文字色を暗くまたは明るくします。

「安全な問題をすべて修正」は数回のパスでまとめて適用します。必要な文字が含まれないフォントのような問題には、自動修正はありません。修正は縦向きレイアウトでも使えます。コマンドラインでは lienzo-ui --open ファイル --review（エラーがあると終了コード 1）。

## Unity からインポート：プレハブと CLI

ファイル ›「Unity からインポート…」は、uGUI の .prefab（Unity のテキスト YAML）または Lienzo UI が書き出した Designs/*.json を読み込み、通常のデザインを作ります。

- 変換できなかった内容は警告として一覧表示されます。
- Unity プロジェクトのフォルダーを検出すると、そのセッション中は画像をそこを基準に解決します。
- ウィンドウなし：lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json（--review や --export も併用可）。
- プロジェクト全体：lienzo-ui --import-unity-project MyProject --out Imported は、すべての UI プレハブを変換します。既存ファイルは上書きせず（_2 を付加）、UI のないプレハブはスキップします。
- 詳細は docs/UNITY_IMPORT.md を参照してください。

## コミュニティ：プラグイン、フォント、ルール

「コミュニティ」タブでは、「インストール」を押すだけで、プラグイン、フォント、画像、テンプレート、要素、テーマといった共有コンテンツを追加できます。

- コミュニティのプラグインは community-<名前>-<ハッシュ> という名前のフォルダーに入ります。まず一時フォルダーに書き込むので、中断されたインストールで中途半端なプラグインが残ることはありません。「プラグイン」タブに表示されます。
- スクリプトは同じ Rhai スクリプトです。サンドボックス内で動き、ファイルやネットワークにはアクセスできず、演算回数・時間・メモリに上限があります（docs/PLUGINS.md）。
- インストールしたフォントは「テキスト」タブに追加され、デザインと一緒に Unity へエクスポートされます。
- ダウンロードは公開された SHA-256 フィンガープリントで検証され、一致しなければ何もインストールされません。
- 公開するには、サインイン（GitHub または Google）し、公開名を決めてルールに同意します。すべて事前に審査され、18禁コンテンツは明示が必要です。docs/TERMS.md、docs/COMMUNITY_GUIDELINES.md、docs/PRIVACY.md を参照してください。

インストール前に、「プレビュー」でフォントのサンプルテキストや、プラグインのコマンド、エクスポーター、ファイル、ソースコード（読み取り専用）を確認できます。何もインストールされず、ダウンロード数にも数えられません。「翻訳」は、押したときだけその説明を無料の MyMemory サービスに送信します（docs/PRIVACY.md）。

## 自動保存、復元、元に戻す

- 編集中は約1分ごとに、デザイン全体（縦向きレイアウトを含む）がアプリの復旧フォルダーに書き込まれます。テストモード中は書き込まれません。復旧データは最大10件まで保持されます。
- 起動時、プロジェクトファイルより新しい復旧データがあれば、「復元」か「破棄」を選べます。通常の保存をすれば不要になります。
- 「アクセシビリティ › 自動保存」でオフにできます。
- 元に戻す履歴は200ステップ、約64 MB までに制限されています。古いステップが破棄されると、ステータスバーに「履歴を整理しました」と表示されます。
- プラグインのコマンド1回、そして「チェック」ウィンドウの「修正」1回は、それぞれ元に戻す1ステップです。
