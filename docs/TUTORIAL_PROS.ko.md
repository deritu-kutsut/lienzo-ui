# 전문가 튜토리얼

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROFESIONALES.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

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

- **편집기에서**: 테스트 모드에서 각 요소는 클릭 시 Rhai 스크립트를 실행할 수 있습니다(`mostrar("Name")`, `ocultar`, `alternar`, `texto`, `color`, `mover`…). 흐름을 시험해 보기에 좋습니다. 테스트 중 바꾼 내용은 테스트 모드를 나갈 때 버려지며 저장되거나 내보내지지 않습니다.
- **Unity에서**: `<Design>View`는 `<Design>ViewBase`를 상속하며, 버튼, 슬라이더, 토글, 입력 필드마다 필드가 하나씩 있고, 재정의할 `On<Element>Click` / `On<Element>Changed` 메서드가 있습니다.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

상호작용 요소에는 고유한 이름을 붙이세요: 스크립트와 `GameObject.Find`는 일치하는 것 중 첫 번째만 찾습니다. **검토**는 중복된 이름을 경고합니다.

## 디자인 검토

**검토** 버튼(및 명령줄의 `--revisar`)은 기준 해상도에서 다음을 확인합니다:

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
lienzo-ui --plantillas                                   # 템플릿 목록 표시 (0-12)
lienzo-ui --plantilla 4 --guardar inventario.editorgui.json
lienzo-ui --abrir inventario.editorgui.json --revisar    # 오류가 있으면 종료 코드 1
lienzo-ui --abrir inventario.editorgui.json --exportar "C:/MyGame"   # 창 없이
lienzo-ui --abrir inventario.editorgui.json --screenshot capture.png
lienzo-ui --probar 1080x2400                             # 해당 화면으로 테스트 모드에서 엽니다
lienzo-ui --tutorial [--profesional]                     # 튜토리얼을 강제로 엽니다
lienzo-ui --sin-tutorial                                 # 튜토리얼을 열지 않습니다
```

`--guardar`, `--exportar`, `--revisar`는 창을 열지 않습니다. 스크립트, CI 및 [MCP 서버](MCP.md)(`revisar_diseno` 도구)에서 사용하세요.

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

확장하려면: [EXTENDER.md](EXTENDER.md).
