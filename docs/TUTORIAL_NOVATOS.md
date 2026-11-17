# Tutorial para novatos

Tu primera interfaz con Lienzo UI y tu primera exportación a Unity. No necesitas saber programar.

## 1. Instalar y abrir

1. Descarga `LienzoUI-windows.zip` desde las releases de GitHub o desde itch.io.
2. Descomprímelo y abre `lienzo-ui.exe`.
3. La primera vez se abre el **tutorial** (este mismo contenido, dentro de la app). Al terminarlo, o con «Saltar e ir al editor», se abre el editor. Para volver a verlo: botón **Tutorial** de la barra superior o **F1**.

![Tutorial dentro de la app](imagenes/tutorial.png)

## 2. La pantalla

| Zona | Qué hay |
|---|---|
| Barra superior | Archivo, Editar, Ver, Lienzo, **Probar**, nombre del diseño, **Tutorial**, **Guardar**, **Código…**, **Exportar a Unity** |
| Barra izquierda | **Jerarquía** (capas), **Elementos**, **Plantillas**, **Texto**, **Imágenes** |
| Centro | El lienzo: la pantalla de tu juego (1920 × 1080 por defecto) |
| Derecha | El **Inspector**: propiedades de lo seleccionado |

![Editor](imagenes/editor.png)

## 3. Empieza con una plantilla

En **Plantillas** elige Menú principal, HUD de juego, Ventana de diálogo, Ajustes o Inventario. Todo lo que trae se puede cambiar. Para empezar de cero: «En blanco».

## 4. Edita

- Arrastra elementos desde **Elementos** al lienzo.
- Clic selecciona; arrastrar mueve; las esquinas cambian el tamaño; el tirador superior gira.
- Doble clic en un texto para escribir.
- En el **Inspector** cambia colores, bordes, sombras, fuentes y anclas.

| Atajo | Acción |
|---|---|
| Ctrl+Z / Ctrl+Y | Deshacer / rehacer |
| Ctrl+D / Supr | Duplicar / borrar |
| Ctrl+G / Ctrl+Shift+G | Agrupar / desagrupar |
| Flechas (Shift) | Mover 1 px (10 px) |
| Ctrl+S | Guardar |
| F5 | Probar |
| Ctrl+E | Exportar a Unity |
| F1 | Tutorial |

## 5. Prueba

Pulsa **Probar** (F5): la interfaz se vuelve interactiva y puedes elegir otras pantallas (móvil, 4:3, ultrapanorámica) para ver cómo se adapta. Esc sale.

## 6. Exporta a Unity

1. Pulsa **Exportar a Unity** (Ctrl+E). La primera vez elige la carpeta de tu proyecto (la que contiene `Assets`).
2. Vuelve a Unity: el diseño aparece como prefab en `Assets/EditorGUI/Prefabs`.
3. Arrástralo a la escena, o usa **Tools › Lienzo UI › Importar diseño en la escena**.

Cada vez que vuelvas a exportar, Unity actualiza el prefab solo.

## Siguientes pasos

- [Tutorial para profesionales](TUTORIAL_PROFESIONALES.md): qué genera la exportación y cómo poner tu lógica.
- [Servidor MCP](MCP.md): pide cambios a una IA en lenguaje natural.
