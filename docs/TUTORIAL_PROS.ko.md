# 전문가 튜토리얼

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

## 내보내기가 생성하는 파일

모든 파일은 Unity 프로젝트의 `Assets/EditorGUI`에 기록됩니다:

| 파일 | 용도 |
|---|---|
| `Designs/<Design>.json` | 디자인입니다. 변경될 때마다 가져오기 도구가 프리팹을 다시 만듭니다. |
| `Editor/EditorGuiImporter.cs` | uGUI + TextMeshPro로 프리팹을 만듭니다. **Tools › Lienzo UI** 메뉴. |
| `Prefabs/<Design>.prefab` | 바로 사용할 수 있습니다. |
| `UI Toolkit/<Design>.uxml` + `.uss` | UI Toolkit 버전 (UIDocument, PanelSettings 1920×1080 Scale With Screen Size). |
| `Scripts/<Design>OnGUI.cs` | IMGUI 버전: GameObject에 추가하세요. |
| `Scripts/<Design>ViewBase.cs` | 형식이 지정된 참조와 이벤트 (다시 생성됨). |
| `Scripts/<Design>View.cs` | **여러분의** 로직: 한 번만 만들어지며 절대 덮어쓰지 않습니다. |
| `Sprites/`, `Fonts/` | 디자인에 사용되는 이미지와 글꼴. |

`<Design>`은 제목에서 액센트를 뺀 PascalCase 이름입니다 (`Menú principal` → `MenuPrincipal`).

이미지가 없으면 아무것도 기록되지 않습니다. 내보내기는 프로젝트를 건드리기 전에 멈추고, 읽을 수 없었던 파일을 알려 줍니다.

## 앵커와 크기 조절

- 아무것도 선택하지 않으면 인스펙터에 **Canvas Scaler**가 표시됩니다: 기준 해상도, *match*, 배경.
- 각 요소는 `RectTransform`처럼 앵커, 피벗, 오프셋을 가집니다. 기준 해상도가 바뀌면 요소는 앵커를 따릅니다.
- 기본 템플릿은 1920×1080 기준으로 설계되었습니다. 더 작은 화면이나 세로 페이지에서는 고르게 축소되어 맞춰집니다.
- 내보내기 전에 여러 화면에서 **테스트**(F5)로 확인하세요.

## 레이아웃과 목록

컨테이너는 자식 요소를 가로, 세로 또는 격자로 배치할 수 있습니다(Unity의 *Layout Groups*처럼). 간격과 여백도 설정합니다. ScrollView는 내용을 계산합니다. 계층 구조의 순서가 그리기 순서입니다.

## 로직

- **편집기에서**: 테스트 모드에서 각 요소는 클릭 시 Rhai 스크립트를 실행할 수 있습니다(`show("Name")`, `hide`, `toggle`, `set_text`, `set_color`, `move_by`…). 흐름을 시험해 보기에 좋습니다. 테스트 중 바꾼 내용은 테스트 모드를 나갈 때 버려지며 저장되거나 내보내지지 않습니다.
- **Unity에서**: `<Design>View`는 `<Design>ViewBase`를 상속하며, 버튼, 슬라이더, 토글, 입력 필드마다 필드가 하나씩 있고, 재정의할 `On<Element>Click` / `On<Element>Changed` 메서드가 있습니다.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

상호작용 요소에는 고유한 이름을 붙이세요: 스크립트와 `GameObject.Find`는 일치하는 것 중 첫 번째만 찾습니다. **검토**는 중복된 이름을 경고합니다.

## 디자인 검토

**검토** 버튼(및 명령줄의 `--review`)은 기준 해상도에서 다음을 확인합니다:

| 검사 | 규칙 |
|---|---|
| 글자 크기 | 1080p에서 최소 18px |
| 대비 | WCAG: 4.5:1, 큰 글자(24px, 또는 굵은 18.7px)는 3:1; 비활성 컨트롤은 제외 |
| 터치 크기 | 버튼, 토글, 입력 필드, 드롭다운은 1080p에서 짧은 쪽이 최소 64px |
| 페이지 | 페이지 안팎으로 일부라도 벗어난 요소 |
| 이름 | 상호작용 요소의 중복된 이름 |
| 이미지 | 더 이상 존재하지 않는 이미지 파일 |

대비가 2:1 미만이거나, 요소가 페이지를 완전히 벗어나거나, 이미지가 없으면 오류입니다. 나머지는 경고입니다.

## 명령줄

```bash
lienzo-ui --templates                                   # 템플릿 목록 표시 (0-12)
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # 오류가 있으면 종료 코드 1
lienzo-ui --open inventory.editorgui.json --export "C:/MyGame"   # 창 없이
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # 해당 화면으로 테스트 모드에서 엽니다
lienzo-ui --tutorial [--pro]                     # 튜토리얼을 강제로 엽니다
lienzo-ui --no-tutorial                                 # 튜토리얼을 열지 않습니다
```

`--save`, `--export`, `--review`는 창을 열지 않습니다. 스크립트, CI 및 [MCP 서버](MCP.md)(`review_design` 도구)에서 사용하세요.

--export에 --dry-run을 붙이면 아무것도 쓰지 않고 각 파일을 new, update, same, remove로 나열합니다.

export_unity는 AI가 지정한 프로젝트의 Assets/EditorGUI에 직접 기록하며, 편집기에는 확인 창이 뜨지 않습니다. 승인은 MCP 클라이언트의 도구 승인으로 이루어집니다. AI에게 먼저 dry_run: true로 호출하게 하고, 목록을 확인한 뒤 실제로 내보내세요.

## 소스 코드

Rust + [egui](https://github.com/emilk/egui):

| 파일 | 내용 |
|---|---|
| `src/model.rs` | 데이터: `Doc`, `Element`, `Kind`, 앵커, 레이아웃, 템플릿 |
| `src/canvas.rs` | 캔버스: 선택, 손잡이, 가이드, 확대/축소 |
| `src/panels.rs` | 상단 막대, 사이드바, 인스펙터, 창 |
| `src/app.rs` | 상태, 실행 취소, 명령, 단축키 |
| `src/render.rs` | 요소와 글꼴 그리기 |
| `src/revision.rs` | 디자인 검토 검사 |
| `src/export/` | 내보내기 도구: `ugui.rs`, `uitk.rs`, `imgui.rs`, `view.rs` 및 Unity 가져오기 도구 |
| `src/tutorial.rs` | 내장 튜토리얼 (스페인어 및 영어 페이지) |
| `src/script.rs` | 테스트 모드용 Rhai 스크립트 |

확장하려면: [EXTENDING.md](EXTENDING.md).

## 세로 레이아웃 자세히

세로 레이아웃은 같은 디자인의 두 번째 보기입니다(docs/FORMAT.md › PortraitLayout).

- 자체 기준 해상도인 「세로 크기」가 있습니다(기본값 1080 × 1920).
- 재정의가 없는 요소는 가로 RectTransform을 유지하고 세로 페이지를 기준으로 다시 계산됩니다. 전체 화면 배경은 늘어나고, 오른쪽에 앵커된 요소는 오른쪽 가장자리를 따라갑니다.
- 세로 보기에서 바꾼 내용만 재정의로 저장됩니다(위치, 크기, 회전, 앵커, 표시 여부, 글꼴 크기, 레이아웃). 마우스 오른쪽 버튼 ›「가로로 되돌리기」로 하나를 제거할 수 있습니다.
- 내보내기: JSON에 재정의가 들어 있고, 생성된 LienzoOrientationLayout 스크립트가 Screen.width < Screen.height일 때 이를 적용하며 크기가 바뀌면 다시 적용합니다.
- 「세로 레이아웃 제거」는 모든 재정의를 삭제합니다.

템플릿에는 세로 레이아웃이 이미 들어 있으며 가로 레이아웃의 덮어쓰기로 저장되므로, 세로 보기는 처음부터 휴대폰에 맞춘 레이아웃을 보여 줍니다.

## 9-slice와 기본 제공 팩

늘어나는 프레임, 버튼, 패널을 위한 기능입니다.

- 이미지를 선택하고 「이미지 설정 (Unity)」에서 「9-slice」를 설정하세요. 원본 이미지 픽셀 단위의 테두리(왼쪽, 아래, 오른쪽, 위)이며 Unity의 Sprite Editor와 같습니다. 내보내면 9-slice Sprite가 생성됩니다.
- 「단위당 픽셀 수」: 100이면 테두리가 픽셀 크기 그대로 유지되고, 200이면 절반이 됩니다.
- 기본 제공 팩(assets/pack/manifest.json, docs/CONTENT.md)에는 분할 값과 단위당 픽셀 수가 이미 설정되어 있습니다. 프레임은 512×512에 분할 96, 버튼은 512×192입니다.
- 항목을 사용하면 pack_<id>.<ext> 이름으로 라이브러리에 복사되고(「내 이미지」에서는 숨김), 내보낼 때 Unity의 Sprites로 복사됩니다.
- 「픽셀 RPG HUD」와 「픽셀 메인 메뉴」 템플릿은 픽셀 팩을 사용합니다. Point 필터를 쓰려면 「픽셀 아트」를 켜세요.

## 검토와 빠른 수정

검토 창에는 오류와 경고가 나열되며, 하나를 클릭하면 해당 요소가 선택됩니다. 여섯 가지 문제에는 「수정」 버튼이 있습니다(실행 취소 한 단계).

- 최소 크기보다 작은 글자: 최소 크기로 키웁니다.
- 터치하기에 너무 작은 컨트롤: 중심을 기준으로 키웁니다.
- 페이지 밖으로 일부 나간 요소: 들어갈 수 있으면 안쪽으로 옮깁니다.
- 이름 중복: _2, _3…을 덧붙입니다.
- 빠진 이미지: 참조를 제거합니다.
- 낮은 대비: 글자 색을 어둡게 또는 밝게 바꿉니다.

「안전한 문제 모두 수정」은 몇 차례에 걸쳐 모두 적용합니다. 필요한 글자가 없는 글꼴 같은 문제는 자동 수정이 없습니다. 수정은 세로 레이아웃에서도 동작합니다. 명령줄: lienzo-ui --open 파일 --review (오류가 있으면 종료 코드 1).

## Unity에서 가져오기: 프리팹과 CLI

파일 ›「Unity에서 가져오기…」는 uGUI .prefab(Unity의 텍스트 YAML) 또는 Lienzo UI가 내보낸 Designs/*.json을 읽어 일반 디자인으로 만듭니다.

- 변환하지 못한 내용은 경고로 나열됩니다.
- Unity 프로젝트 폴더를 감지하면 그 세션 동안 이미지를 해당 폴더 기준으로 찾습니다.
- 창 없이: lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json (--review, --export와 함께도 사용 가능).
- 프로젝트 전체: lienzo-ui --import-unity-project MyProject --out Imported는 모든 UI 프리팹을 변환합니다. 기존 파일을 덮어쓰지 않으며(_2를 붙임) UI가 없는 프리팹은 건너뜁니다.
- 자세한 내용은 docs/UNITY_IMPORT.md를 참고하세요.

## 커뮤니티: 플러그인, 글꼴, 규칙

커뮤니티 탭에서 「설치」를 누르면 플러그인, 글꼴, 이미지, 템플릿, 요소, 테마 같은 공유 콘텐츠를 한 번에 추가합니다.

- 커뮤니티 플러그인은 community-<이름>-<해시>라는 폴더에 들어갑니다. 먼저 임시 폴더에 기록하므로 중단된 설치가 반쪽짜리 플러그인을 남기지 않습니다. 플러그인 탭에 나타납니다.
- 스크립트는 같은 Rhai 스크립트입니다. 샌드박스에서 실행되어 파일이나 네트워크에 접근할 수 없고, 연산 횟수, 시간, 메모리에 제한이 있습니다(docs/PLUGINS.md).
- 설치한 글꼴은 텍스트 탭에 추가되고 디자인과 함께 Unity로 내보내집니다.
- 다운로드는 공개된 SHA-256 지문과 대조하며, 일치하지 않으면 아무것도 설치하지 않습니다.
- 게시하려면 로그인(GitHub 또는 Google)하고, 공개 이름을 정하고, 규칙에 동의해야 합니다. 모든 것이 먼저 검토되며 +18 콘텐츠는 표시해야 합니다. docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md, docs/PRIVACY.md를 참고하세요.

설치하기 전에 「미리보기」로 글꼴의 예시 텍스트, 또는 플러그인의 명령, 내보내기, 파일, 소스 코드(읽기 전용)를 볼 수 있습니다. 아무것도 설치되지 않고 다운로드로 집계되지도 않습니다. 「번역」은 누를 때만 그 설명을 무료 MyMemory 서비스로 보냅니다(docs/PRIVACY.md).

## 자동 저장, 복구, 실행 취소

- 편집하는 동안 약 1분마다 디자인 전체(세로 레이아웃 포함)가 앱의 복구 폴더에 기록됩니다. 테스트 모드에서는 기록하지 않습니다. 복구본은 최대 10개까지 유지됩니다.
- 시작할 때 프로젝트 파일보다 새로운 복구본이 있으면 「복원」 또는 「삭제」를 고를 수 있습니다. 정상적으로 저장하면 더 이상 필요하지 않습니다.
- 「접근성 › 자동 저장」에서 끌 수 있습니다.
- 실행 취소 기록은 200단계와 약 64 MB로 제한됩니다. 가장 오래된 단계가 삭제되면 상태 표시줄에 「기록 정리됨」이라고 표시됩니다.
- 플러그인 명령 하나, 그리고 검토 창의 「수정」 한 번은 각각 실행 취소 한 단계입니다.
