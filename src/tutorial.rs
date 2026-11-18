//! Built-in tutorial with two tracks (beginners and professionals).
//! It opens by itself the first time Lienzo UI starts; when finished or skipped, the editor remains.
//! The "Tutorial" button in the top bar or F1 reopens it.
//! To add a page, add an element to `BEGINNER` and `PRO` (English) and to `BEGINNER_ES` and `PRO_ES` (Spanish), plus a page in each `assets/tutorial/<code>.json`.

use crate::app::EditorApp;
use crate::theme::{ACCENT, LINE, MUTED, PANEL, TEXT};
use eframe::egui::{self, Align, Color32, Layout, RichText, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    Beginner,
    Pro,
}

pub struct Page {
    pub title: &'static str,
    pub text: &'static str,
}

/// State of the open tutorial.
#[derive(Clone, Copy, Debug)]
pub struct Tutorial {
    pub track: Track,
    pub page: usize,
}

pub const BEGINNER_ES: &[Page] = &[
    Page {
        title: "Bienvenido a Lienzo UI",
        text: "Lienzo UI es un editor visual para diseñar las interfaces de tu juego (menús, HUD, ventanas, inventarios) \
como en una herramienta de diseño, y llevarlas a Unity con un clic.\n\n\
Al exportar obtienes la misma interfaz en uGUI con TextMeshPro, en UI Toolkit (UXML + USS) y en IMGUI.\n\n\
Al terminar este tutorial se abre el editor. Puedes volver aquí cuando quieras con el botón «Tutorial» de la barra superior o con F1.",
    },
    Page {
        title: "La pantalla del editor",
        text: "• Arriba: los menús Archivo, Editar, Ver y Lienzo, el botón Probar, el nombre del diseño, Accesibilidad, Revisar, Temas, Tutorial, Guardar, Código… y Exportar a Unity.\n\
• A la izquierda, la barra con Jerarquía (las capas), Elementos, Plantillas, Texto, Imágenes, Plugins y Comunidad.\n\
• En el centro, el lienzo: la pantalla de tu juego a la resolución de referencia (1920 × 1080 por defecto).\n\
• A la derecha, el Inspector: las propiedades del elemento seleccionado (posición, anclas, colores, texto…).",
    },
    Page {
        title: "Empieza con una plantilla",
        text: "Abre Plantillas en la barra izquierda y elige una de las 12: menú principal, HUD, pausa, fin de partida, selección de nivel, inventario, diálogo de personaje, clasificación…\n\n\
Es la forma más rápida de ver cómo se construye una interfaz completa. Todo lo que trae se puede cambiar.\n\n\
Si prefieres empezar de cero, elige «En blanco» o Archivo › Nuevo diseño en blanco.",
    },
    Page {
        title: "Añadir y editar elementos",
        text: "• Arrastra elementos desde Elementos al lienzo: paneles, textos, botones, casillas, sliders, campos de texto, desplegables, listas con scroll e imágenes. En «Componentes listos» hay botones, barras de vida y maná, tarjetas e insignias; en Texto, piezas de HUD como temporizador, monedas o daño.\n\
• Clic selecciona, arrastrar mueve, las esquinas cambian el tamaño y el tirador superior gira.\n\
• Doble clic en un texto para escribir en él.\n\
• Ctrl+Z / Ctrl+Y deshacen y rehacen; Ctrl+D duplica; Supr borra; Ctrl+G agrupa; las flechas mueven 1 px (Shift: 10).\n\
• En el Inspector cambias colores, bordes, sombras, fuentes y las anclas (cómo se adapta el elemento a otras pantallas).",
    },
    Page {
        title: "Probar y exportar a Unity",
        text: "• Probar (F5): el diseño se vuelve interactivo y puedes simular otras pantallas (móvil, 4:3, ultrapanorámica) para ver cómo se adapta.\n\
• Guardar (Ctrl+S): guarda el proyecto .editorgui.json para seguir otro día.\n\
• Revisar: avisa de letra pequeña, poco contraste, botones pequeños para tocar, elementos fuera de la página e imágenes que faltan.\n\
• Exportar a Unity (Ctrl+E): la primera vez elige la carpeta de tu proyecto (la que contiene Assets).\n\n\
En Unity, al volver a la ventana, el diseño aparece como prefab en Assets/EditorGUI/Prefabs. También tienes el menú Tools › Lienzo UI › Importar diseño en la escena.",
    },
];

pub const PRO_ES: &[Page] = &[
    Page {
        title: "Qué genera la exportación",
        text: "Todo va a Assets/EditorGUI de tu proyecto:\n\n\
• Designs/<Diseño>.json: el diseño; el importador rehace el prefab cada vez que cambia.\n\
• Editor/EditorGuiImporter.cs: construye el prefab con uGUI + TextMeshPro.\n\
• Prefabs/<Diseño>.prefab: listo para arrastrar a la escena.\n\
• UI Toolkit/<Diseño>.uxml y .uss: la versión para UIDocument.\n\
• Scripts/<Diseño>OnGUI.cs: la versión IMGUI.\n\
• Scripts/<Diseño>ViewBase.cs (se regenera) y <Diseño>View.cs (tuyo: se crea una vez y nunca se sobrescribe).",
    },
    Page {
        title: "Anclas y escalado",
        text: "El lienzo funciona como un Canvas con Canvas Scaler en modo Scale With Screen Size: referencia, match ancho/alto y fondo se ajustan en el Inspector con nada seleccionado.\n\n\
Cada elemento tiene anclas, pivote y desplazamientos como un RectTransform. Al cambiar la resolución de referencia los elementos siguen sus anclas, igual que en Unity.\n\n\
Usa Probar (F5) con varias pantallas para comprobarlo antes de exportar.",
    },
    Page {
        title: "Layouts, grupos y listas",
        text: "Un contenedor puede ordenar a sus hijos en horizontal, vertical o cuadrícula (como Horizontal/Vertical/Grid Layout Group), con espaciado y relleno.\n\n\
Las listas con scroll (ScrollView) calculan su contenido solas.\n\n\
Agrupa (Ctrl+G) para mover partes enteras y usa la Jerarquía para reordenar capas: el orden de la lista es el orden de dibujo en Unity.",
    },
    Page {
        title: "Lógica: código al hacer clic y vistas",
        text: "En Probar, cada elemento puede ejecutar un pequeño script (Rhai) al hacer clic: mostrar(\"Nombre\"), ocultar, alternar, texto, color, mover… Sirve para prototipar flujos sin Unity.\n\n\
En Unity, <Diseño>View.cs hereda de <Diseño>ViewBase con referencias tipadas a cada botón, slider o campo, y eventos de clic y de valor. Pon ahí la lógica del juego: las reexportaciones no la tocan.",
    },
    Page {
        title: "Línea de comandos y automatización",
        text: "lienzo-ui --templates                          lista las plantillas\n\
lienzo-ui --template 0 --save menu.editorgui.json\n\
lienzo-ui --open menu.editorgui.json --review         revisa el diseño (código 1 si hay errores)\n\
lienzo-ui --open menu.editorgui.json --export <proyecto Unity>\n\
lienzo-ui --open menu.editorgui.json --screenshot captura.png\n\
lienzo-ui --test   1080x2400                    abre en Probar con esa pantalla\n\n\
Con --export, --save y --review no se abre ventana: sirve para scripts, CI y el servidor MCP.",
    },
    Page {
        title: "Servidor MCP y contribuir",
        text: "La carpeta mcp/ trae un servidor MCP sin dependencias (Node 18+): cualquier IA compatible (Claude, Cursor, VS Code…) puede listar plantillas, crear y editar diseños, exportarlos a Unity y sacar capturas.\n\n\
Lienzo UI es de código abierto (MIT). El código está en Rust con egui: src/model.rs (datos), src/canvas.rs (lienzo), src/panels.rs (paneles), src/export/ (Unity).\n\n\
Lee docs/EXTENDER.md para añadir elementos, plantillas o exportadores, y CONTRIBUTING.md para enviar cambios.",
    },
    Page {
        title: "Pixel art",
        text: "Con nada seleccionado, marca «Pixel art» en el Inspector del lienzo:\n\n\
• Las imágenes se dibujan sin suavizado (vecino más cercano).\n\
• Posiciones y tamaños se ajustan a píxeles enteros al soltar.\n\
• En Unity: Canvas «Pixel Perfect» y sprites con filtro Point, sin compresión ni mipmaps.\n\n\
Consejo: usa tamaños múltiplos exactos de tus sprites (16 px → 32, 48, 64…) para que cada píxel se vea igual.",
    },
    Page {
        title: "Plugins",
        text: "La pestaña Plugins de la barra izquierda lista los complementos instalados. Un plugin es una carpeta con plugin.json que puede añadir:\n\n\
• Comandos: scripts Rhai que reciben el diseño y lo devuelven modificado (se deshacen con Ctrl+Z).\n\
• Exportadores: scripts que generan archivos nuevos (por ejemplo, un CSV con todos los textos).\n\
• Plantillas: diseños listos para usar.\n\n\
Los scripts corren aislados, sin acceso a archivos ni red. «Abrir carpeta» lleva a tu carpeta de plugins; la guía está en docs/PLUGINS.md.",
    },
    Page {
        title: "Accesibilidad e idiomas",
        text: "El botón «Accesibilidad» de la barra superior permite:\n\n\
• Cambiar el idioma de la interfaz (español, inglés, portugués, francés, alemán, italiano, ruso, chino, japonés o coreano).\n\
• Agrandar la interfaz hasta el 200 % (también con Ctrl+Alt+= y Ctrl+Alt+-).\n\
• Activar alto contraste: texto blanco y bordes marcados.\n\
• Reducir el movimiento: sin animaciones.\n\n\
Los idiomas son archivos assets/i18n/<código>.tsv: para añadir uno basta con traducir en.tsv.",
    },
];

pub const BEGINNER: &[Page] = &[
    Page {
        title: "Welcome to Lienzo UI",
        text: "Lienzo UI is a visual editor to design your game's interfaces (menus, HUDs, windows, inventories) \
like in a design tool, and bring them to Unity in one click.\n\n\
Exporting gives you the same interface in uGUI with TextMeshPro, in UI Toolkit (UXML + USS) and in IMGUI.\n\n\
When you finish this tutorial the editor opens. Come back any time with the «Tutorial» button in the top bar or F1.",
    },
    Page {
        title: "The editor screen",
        text: "• Top: the File, Edit, View and Canvas menus, the Test button, the design name, Accessibility, Review, Themes, Tutorial, Save, Code… and Export to Unity.\n\
• Left: the bar with Hierarchy (layers), Elements, Templates, Text, Images, Plugins and Community.\n\
• Center: the canvas, your game screen at the reference resolution (1920 × 1080 by default).\n\
• Right: the Inspector, with the properties of the selected element (position, anchors, colors, text…).",
    },
    Page {
        title: "Start from a template",
        text: "Open Templates in the left bar and pick one of the 12: main menu, HUD, pause, game over, level select, inventory, character dialogue, leaderboard…\n\n\
It is the fastest way to see how a complete interface is built. Everything in it can be changed.\n\n\
To start from scratch, choose «Blank» or File › New blank design.",
    },
    Page {
        title: "Add and edit elements",
        text: "• Drag elements from Elements onto the canvas: panels, texts, buttons, toggles, sliders, input fields, dropdowns, scroll lists and images. «Ready-made components» has buttons, health and mana bars, cards and badges; Text has HUD pieces such as timer, coins or damage.\n\
• Click selects, drag moves, corners resize and the top handle rotates.\n\
• Double-click a text to type in it.\n\
• Ctrl+Z / Ctrl+Y undo and redo; Ctrl+D duplicates; Del deletes; Ctrl+G groups; arrows move 1 px (Shift: 10).\n\
• In the Inspector you change colors, borders, shadows, fonts and the anchors (how the element adapts to other screens).",
    },
    Page {
        title: "Test and export to Unity",
        text: "• Test (F5): the design becomes interactive and you can simulate other screens (phone, 4:3, ultrawide) to see how it adapts.\n\
• Save (Ctrl+S): saves the .editorgui.json project to keep working later.\n\
• Review: warns about small text, low contrast, buttons too small to tap, elements outside the page and missing images.\n\
• Export to Unity (Ctrl+E): the first time, choose your project folder (the one that contains Assets).\n\n\
Back in Unity, when you return to the window, the design appears as a prefab in Assets/EditorGUI/Prefabs. There is also the menu Tools › Lienzo UI › Importar diseño en la escena (Import design into the scene).",
    },
];

pub const PRO: &[Page] = &[
    Page {
        title: "What the export generates",
        text: "Everything goes to Assets/EditorGUI in your project:\n\n\
• Designs/<Design>.json: the design; the importer rebuilds the prefab whenever it changes.\n\
• Editor/EditorGuiImporter.cs: builds the prefab with uGUI + TextMeshPro.\n\
• Prefabs/<Design>.prefab: ready to drop into a scene.\n\
• UI Toolkit/<Design>.uxml and .uss: the UIDocument version.\n\
• Scripts/<Design>OnGUI.cs: the IMGUI version.\n\
• Scripts/<Design>ViewBase.cs (regenerated) and <Design>View.cs (yours: created once, never overwritten).",
    },
    Page {
        title: "Anchors and scaling",
        text: "The canvas works like a Canvas with a Canvas Scaler in Scale With Screen Size mode: reference resolution, width/height match and background are set in the Inspector with nothing selected.\n\n\
Each element has anchors, pivot and offsets like a RectTransform. When the reference resolution changes, elements follow their anchors, just like in Unity.\n\n\
Use Test (F5) with several screens before exporting.",
    },
    Page {
        title: "Layouts, groups and lists",
        text: "A container can arrange its children horizontally, vertically or in a grid (like Horizontal/Vertical/Grid Layout Group), with spacing and padding.\n\n\
Scroll lists (ScrollView) compute their content automatically.\n\n\
Group (Ctrl+G) to move whole parts and use the Hierarchy to reorder layers: the list order is the drawing order in Unity.",
    },
    Page {
        title: "Logic: click scripts and views",
        text: "In Test mode each element can run a small Rhai script on click: mostrar(\"Name\"), ocultar, alternar, texto, color, mover… Useful to prototype flows without Unity.\n\n\
In Unity, <Design>View.cs inherits from <Design>ViewBase with typed references to every button, slider or field, and click and value events. Put your game logic there: re-exports never touch it.",
    },
    Page {
        title: "Command line and automation",
        text: "lienzo-ui --templates                          lists the templates\n\
lienzo-ui --template 0 --save menu.editorgui.json\n\
lienzo-ui --open menu.editorgui.json --review         reviews the design (exit code 1 on errors)\n\
lienzo-ui --open menu.editorgui.json --export <Unity project>\n\
lienzo-ui --open menu.editorgui.json --screenshot capture.png\n\
lienzo-ui --test   1080x2400                    opens in Test mode with that screen\n\n\
With --export, --save and --review no window opens: use them for scripts, CI and the MCP server.",
    },
    Page {
        title: "MCP server and contributing",
        text: "The mcp/ folder ships a dependency-free MCP server (Node 18+): any compatible AI (Claude, Cursor, VS Code…) can list templates, create and edit designs, export them to Unity and take screenshots.\n\n\
Lienzo UI is open source (MIT). The code is Rust with egui: src/model.rs (data), src/canvas.rs (canvas), src/panels.rs (panels), src/export/ (Unity).\n\n\
Read docs/EXTENDER.md to add elements, templates or exporters, and CONTRIBUTING.md to send changes.",
    },
    Page {
        title: "Pixel art",
        text: "With nothing selected, tick «Pixel art» in the canvas Inspector:\n\n\
• Images are drawn without smoothing (nearest neighbor).\n\
• Positions and sizes snap to whole pixels when you release.\n\
• In Unity: Canvas «Pixel Perfect» and sprites with Point filter, no compression or mipmaps.\n\n\
Tip: use exact multiples of your sprite size (16 px → 32, 48, 64…) so every pixel looks the same.",
    },
    Page {
        title: "Plugins",
        text: "The Plugins tab in the left bar lists the installed add-ons. A plugin is a folder with a plugin.json that can add:\n\n\
• Commands: Rhai scripts that receive the design and return it modified (undo with Ctrl+Z).\n\
• Exporters: scripts that generate new files (for example, a CSV with every text).\n\
• Templates: ready-made designs.\n\n\
Scripts run sandboxed, without file or network access. «Open folder» takes you to your plugins folder; the guide is docs/PLUGINS.md.",
    },
    Page {
        title: "Accessibility and languages",
        text: "The «Accessibility» button in the top bar lets you:\n\n\
• Change the interface language (English, Spanish, Portuguese, French, German, Italian, Russian, Chinese, Japanese or Korean).\n\
• Enlarge the interface up to 200 % (also Ctrl+Alt+= and Ctrl+Alt+-).\n\
• Turn on high contrast: white text and strong borders.\n\
• Reduce motion: no animations.\n\n\
Languages are assets/i18n/<code>.tsv files: to add one, translate en.tsv.",
    },
];

/// Tutorials for the other languages: `assets/tutorial/<code>.json` with the same pages as `BEGINNER` and `PRO`.
const TUTORIALS: &[(&str, &str)] = &[
    ("pt", include_str!("../assets/tutorial/pt.json")),
    ("fr", include_str!("../assets/tutorial/fr.json")),
    ("de", include_str!("../assets/tutorial/de.json")),
    ("it", include_str!("../assets/tutorial/it.json")),
    ("ru", include_str!("../assets/tutorial/ru.json")),
    ("zh", include_str!("../assets/tutorial/zh.json")),
    ("ja", include_str!("../assets/tutorial/ja.json")),
    ("ko", include_str!("../assets/tutorial/ko.json")),
];

/// Tutorial files. Files with the old Spanish keys (`novato`, `profesional`, `titulo`, `texto`) still load through the aliases.
#[derive(serde::Deserialize)]
struct PageJson {
    #[serde(alias = "titulo")]
    title: String,
    #[serde(alias = "texto")]
    text: String,
}

#[derive(serde::Deserialize)]
struct TutorialJson {
    #[serde(alias = "novato")]
    beginner: Vec<PageJson>,
    #[serde(alias = "profesional")]
    pro: Vec<PageJson>,
}

type Tracks = (&'static [Page], &'static [Page]);

/// Read only once; a malformed or incomplete file is ignored (English is used).
static OTHERS: std::sync::LazyLock<std::collections::HashMap<&'static str, Tracks>> = std::sync::LazyLock::new(|| {
    let leak = |v: Vec<PageJson>| -> &'static [Page] {
        Box::leak(v.into_iter().map(|p| Page { title: Box::leak(p.title.into_boxed_str()), text: Box::leak(p.text.into_boxed_str()) }).collect::<Vec<_>>().into_boxed_slice())
    };
    TUTORIALS
        .iter()
        .filter_map(|(code, json)| {
            let t: TutorialJson = serde_json::from_str(json).ok()?;
            (t.beginner.len() == BEGINNER_ES.len() && t.pro.len() == PRO_ES.len()).then(|| (*code, (leak(t.beginner), leak(t.pro))))
        })
        .collect()
});

pub fn pages(p: Track) -> &'static [Page] {
    let (beginner, pro) = match crate::i18n::code() {
        "es" => (BEGINNER_ES, PRO_ES),
        code => OTHERS.get(code).copied().unwrap_or((BEGINNER, PRO)),
    };
    match p {
        Track::Beginner => beginner,
        Track::Pro => pro,
    }
}

impl EditorApp {
    pub fn open_tutorial(&mut self, track: Track) {
        self.tutorial = Some(Tutorial { track, page: 0 });
    }

    /// Closes the tutorial (the editor remains) and remembers it so it does not open by itself next time.
    pub fn close_tutorial(&mut self) {
        self.tutorial = None;
        if !self.settings.tutorial_seen {
            self.settings.tutorial_seen = true;
            self.settings.save();
        }
    }

    /// Next page; on the last one, closes the tutorial and leaves the editor.
    pub fn advance_tutorial(&mut self) {
        if let Some(t) = &mut self.tutorial {
            if t.page + 1 < pages(t.track).len() {
                t.page += 1;
            } else {
                self.close_tutorial();
            }
        }
    }

    pub fn tutorial_window(&mut self, ctx: &egui::Context) {
        let Some(t) = self.tutorial else { return };
        let list = pages(t.track);
        let page = t.page.min(list.len() - 1);
        let last = page + 1 == list.len();

        // Keyboard: arrows turn pages, Enter advances, Esc closes.
        let (left, der, intro, esc) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft),
                i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Enter),
                i.consume_key(egui::Modifiers::NONE, egui::Key::Escape),
            )
        });

        let mut action: Option<u8> = None; // 0 previous, 1 next, 2 close, 3 beginner, 4 pro
        #[allow(deprecated)]
        let id = egui::Id::new("tutorial");
        let frame = egui::Frame::new().fill(PANEL()).corner_radius(12.0).inner_margin(22.0).stroke(egui::Stroke::new(1.0, LINE()));
        let modal = egui::Modal::new(id).frame(frame).show(ctx, |ui| {
            ui.set_width(760.0);
            ui.horizontal(|ui| {
                for (p, name, a) in [(Track::Beginner, "Beginner tutorial", 3u8), (Track::Pro, "Pro tutorial", 4u8)] {
                    let sel = t.track == p;
                    let txt = RichText::new(crate::i18n::t(name)).size(14.5).color(if sel { ACCENT() } else { MUTED() });
                    if ui.add(egui::Button::selectable(sel, txt)).clicked() {
                        action = Some(a);
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(crate::i18n::t("Skip and go to editor")).clicked() {
                        action = Some(2);
                    }
                });
            });
            ui.add(egui::Separator::default().spacing(14.0));
            let track = crate::i18n::t(if t.track == Track::Beginner { "BEGINNERS" } else { "PROFESSIONALS" });
            ui.label(RichText::new(format!("{} · {} {} / {}", track, crate::i18n::t("STEP"), page + 1, list.len())).size(11.5).strong().color(ACCENT()));
            ui.add_space(4.0);
            ui.label(RichText::new(list[page].title).size(24.0).strong().color(TEXT()));
            ui.add_space(10.0);
            egui::ScrollArea::vertical().max_height(300.0).min_scrolled_height(300.0).show(ui, |ui| {
                ui.set_min_height(300.0);
                ui.label(RichText::new(list[page].text).size(15.0).color(TEXT()));
            });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.add_enabled(page > 0, egui::Button::new(crate::i18n::t("Previous")).min_size(Vec2::new(96.0, 30.0))).clicked() {
                    action = Some(0);
                }
                let width = ui.available_width();
                let points = list.len() as f32 * 14.0;
                ui.add_space(((width - points - 170.0) / 2.0).max(0.0));
                for i in 0..list.len() {
                    let (r, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.0, if i == page { ACCENT() } else { LINE() });
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let text = crate::i18n::t(if last { "Open the editor" } else { "Next" });
                    let b = egui::Button::new(RichText::new(text).strong().color(Color32::WHITE)).fill(ACCENT()).min_size(Vec2::new(130.0, 30.0));
                    if ui.add(b).clicked() {
                        action = Some(1);
                    }
                });
            });
        });
        if modal.should_close() || esc {
            action = Some(2);
        }
        if der || intro {
            action = Some(1);
        }
        if left && page > 0 {
            action = Some(0);
        }
        match action {
            Some(0) => {
                if let Some(t) = &mut self.tutorial {
                    t.page = t.page.saturating_sub(1);
                }
            }
            Some(1) => self.advance_tutorial(),
            Some(2) => self.close_tutorial(),
            Some(3) => self.open_tutorial(Track::Beginner),
            Some(4) => self.open_tutorial(Track::Pro),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_tracks_have_content() {
        assert_eq!(BEGINNER.len(), BEGINNER_ES.len(), "the Spanish beginner track must have the same pages");
        assert_eq!(PRO.len(), PRO_ES.len(), "the Spanish pro track must have the same pages");
        assert!(BEGINNER_ES.len() >= 3 && PRO_ES.len() >= 3);
        for p in BEGINNER_ES.iter().chain(PRO_ES) {
            assert!(!p.title.trim().is_empty() && !p.text.trim().is_empty());
        }
    }

    #[test]
    fn every_language_has_a_complete_tutorial() {
        for (code, json) in TUTORIALS {
            let t: TutorialJson = serde_json::from_str(json).unwrap_or_else(|e| panic!("{}.json is not valid: {}", code, e));
            assert_eq!(t.beginner.len(), BEGINNER_ES.len(), "{}: beginner pages", code);
            assert_eq!(t.pro.len(), PRO_ES.len(), "{}: pro pages", code);
            for p in t.beginner.iter().chain(&t.pro) {
                assert!(!p.title.trim().is_empty() && !p.text.trim().is_empty(), "{}: empty page", code);
            }
            assert!(OTHERS.contains_key(code), "{} was not loaded", code);
        }
        // Every interface language has its own tutorial (Spanish and English are in the code).
        for (code, _, _) in crate::i18n::LANGUAGES {
            assert!(matches!(*code, "es" | "en") || TUTORIALS.iter().any(|(c, _)| c == code), "{} has no tutorial", code);
        }
    }

    #[test]
    fn old_spanish_tutorial_keys_still_load() {
        let old = r#"{ "novato": [ { "titulo": "A", "texto": "B" } ], "profesional": [ { "titulo": "C", "texto": "D" } ] }"#;
        let t: TutorialJson = serde_json::from_str(old).expect("a tutorial file with Spanish keys must load");
        assert_eq!((t.beginner[0].title.as_str(), t.beginner[0].text.as_str(), t.pro[0].title.as_str()), ("A", "B", "C"));
        let new = r#"{ "beginner": [ { "title": "A", "text": "B" } ], "pro": [ { "title": "C", "text": "D" } ] }"#;
        assert!(serde_json::from_str::<TutorialJson>(new).is_ok());
    }
}
