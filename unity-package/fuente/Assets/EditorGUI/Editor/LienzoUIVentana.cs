// Lienzo UI for Unity · access window. MIT.
// Opens the Lienzo UI app (visual interface editor) and shows how to bring its designs into Unity.
// The desktop app is free and open source: https://github.com/deritu-kutsut/lienzo-ui
#if UNITY_EDITOR
using System;
using System.Diagnostics;
using System.IO;
using UnityEditor;
using UnityEngine;
using Debug = UnityEngine.Debug;

namespace EditorGui
{
    public class LienzoUIVentana : EditorWindow
    {
        const string PrefExe = "EditorGui.RutaLienzoUI";
        const string UrlItch = "https://deritu.itch.io/lienzo-ui";
        const string UrlDescarga = "https://github.com/deritu-kutsut/lienzo-ui/releases/latest";
        const string UrlDocs = "https://github.com/deritu-kutsut/lienzo-ui#readme";
        const string CarpetaMuestras = EditorGuiImporter.Root + "/Samples";

        string exe;
        Vector2 scroll;

        static bool Es { get { return Application.systemLanguage == SystemLanguage.Spanish; } }
        static string T(string es, string en) { return Es ? es : en; }

        [MenuItem("Tools/Lienzo UI/Open Lienzo UI (app) · Abrir la app")]
        static void Abrir()
        {
            var w = GetWindow<LienzoUIVentana>("Lienzo UI");
            w.minSize = new Vector2(400, 380);
            w.Show();
        }

        void OnEnable()
        {
            exe = EditorPrefs.GetString(PrefExe, "");
            if (string.IsNullOrEmpty(exe) || !File.Exists(exe)) exe = Buscar();
        }

        /// <summary>Looks for the app in the usual Windows locations.</summary>
        static string Buscar()
        {
            var candidatos = new[]
            {
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Programs", "Lienzo UI", "Lienzo UI.exe"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles), "Lienzo UI", "Lienzo UI.exe"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments), "Lienzo UI", "Lienzo UI.exe"),
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Desktop), "Lienzo UI", "Lienzo UI.exe"),
            };
            foreach (var c in candidatos) if (File.Exists(c)) return c;
            return "";
        }

        static void Lanzar(string ruta, string argumentos)
        {
            try
            {
                var info = new ProcessStartInfo(ruta, argumentos) { UseShellExecute = false, WorkingDirectory = Path.GetDirectoryName(ruta) };
                Process.Start(info);
            }
            catch (Exception e)
            {
                Debug.LogWarning("[Lienzo UI] " + T("No se pudo abrir la app: ", "Could not open the app: ") + e.Message);
            }
        }

        void OnGUI()
        {
            scroll = EditorGUILayout.BeginScrollView(scroll);
            GUILayout.Space(6);
            EditorGUILayout.LabelField("Lienzo UI", EditorStyles.boldLabel);
            EditorGUILayout.HelpBox(T(
                "Lienzo UI es un editor visual de interfaces para juegos: diseñas una pantalla y la exportas a Unity como uGUI + TextMeshPro, UI Toolkit e IMGUI. Es una app de escritorio gratuita y de código abierto (Windows).",
                "Lienzo UI is a visual editor for game interfaces: design a screen and export it to Unity as uGUI + TextMeshPro, UI Toolkit and IMGUI. It is a free, open source desktop app (Windows)."), MessageType.Info);

            GUILayout.Space(4);
            EditorGUILayout.LabelField(T("1 · La app", "1 · The app"), EditorStyles.boldLabel);
            if (Application.platform != RuntimePlatform.WindowsEditor)
                EditorGUILayout.HelpBox(T("La app oficial es para Windows. En macOS o Linux puedes compilarla desde el código fuente (Rust).", "The official app is for Windows. On macOS or Linux you can build it from source (Rust)."), MessageType.Warning);
            EditorGUILayout.BeginHorizontal();
            EditorGUILayout.TextField(exe);
            if (GUILayout.Button(T("Buscar…", "Browse…"), GUILayout.Width(80)))
            {
                var p = EditorUtility.OpenFilePanel(T("Elige Lienzo UI.exe", "Pick Lienzo UI.exe"), "", "exe");
                if (!string.IsNullOrEmpty(p)) { exe = p; EditorPrefs.SetString(PrefExe, p); }
            }
            EditorGUILayout.EndHorizontal();
            var hay = !string.IsNullOrEmpty(exe) && File.Exists(exe);
            using (new EditorGUI.DisabledScope(!hay))
            {
                if (GUILayout.Button(T("Abrir Lienzo UI", "Open Lienzo UI"), GUILayout.Height(30))) Lanzar(exe, "");
            }
            if (!hay)
            {
                EditorGUILayout.HelpBox(T("No encuentro la app. Descárgala (gratis) y elige su ejecutable.", "The app was not found. Download it (free) and pick its executable."), MessageType.None);
                if (GUILayout.Button(T("Descargar Lienzo UI (GitHub)", "Download Lienzo UI (GitHub)"))) Application.OpenURL(UrlDescarga);
                if (GUILayout.Button("itch.io")) Application.OpenURL(UrlItch);
            }

            GUILayout.Space(8);
            EditorGUILayout.LabelField(T("2 · Cómo traer un diseño a Unity", "2 · Bring a design into Unity"), EditorStyles.boldLabel);
            EditorGUILayout.HelpBox(T(
                "En la app pulsa «Exportar a Unity» (Ctrl+E) y elige esta carpeta de proyecto (la que contiene Assets). Al volver aquí, el prefab aparece solo en Assets/EditorGUI/Prefabs. También: Tools > Lienzo UI.",
                "In the app press \"Export to Unity\" (Ctrl+E) and choose this project folder (the one that contains Assets). Back here, the prefab appears by itself in Assets/EditorGUI/Prefabs. Also: Tools > Lienzo UI."), MessageType.None);
            var proyecto = Directory.GetParent(Application.dataPath).FullName;
            EditorGUILayout.SelectableLabel(proyecto, EditorStyles.textField, GUILayout.Height(18));
            if (GUILayout.Button(T("Copiar la ruta de este proyecto", "Copy this project's path"))) EditorGUIUtility.systemCopyBuffer = proyecto;
            if (GUILayout.Button(T("Regenerar todos los prefabs", "Regenerate all prefabs"))) EditorApplication.ExecuteMenuItem("Tools/Lienzo UI/Regenerar todos los prefabs");
            if (GUILayout.Button(T("Importar un diseño en la escena…", "Import a design into the scene…"))) EditorApplication.ExecuteMenuItem("Tools/Lienzo UI/Importar diseño en la escena…");

            GUILayout.Space(8);
            EditorGUILayout.LabelField(T("3 · Proyectos de ejemplo", "3 · Sample projects"), EditorStyles.boldLabel);
            var carpeta = Path.Combine(proyecto, CarpetaMuestras);
            if (Directory.Exists(carpeta))
            {
                foreach (var f in Directory.GetFiles(carpeta, "*.editorgui.json"))
                {
                    EditorGUILayout.BeginHorizontal();
                    EditorGUILayout.LabelField(Path.GetFileName(f).Replace(".editorgui.json", ""));
                    using (new EditorGUI.DisabledScope(!hay))
                    {
                        if (GUILayout.Button(T("Abrir en Lienzo UI", "Open in Lienzo UI"), GUILayout.Width(150))) Lanzar(exe, "--abrir \"" + f + "\"");
                    }
                    EditorGUILayout.EndHorizontal();
                }
            }
            else EditorGUILayout.LabelField(T("No hay ejemplos en este proyecto.", "There are no samples in this project."));

            GUILayout.Space(8);
            EditorGUILayout.BeginHorizontal();
            if (GUILayout.Button(T("Documentación", "Documentation"))) Application.OpenURL(UrlDocs);
            if (GUILayout.Button("GitHub")) Application.OpenURL("https://github.com/deritu-kutsut/lienzo-ui");
            EditorGUILayout.EndHorizontal();
            EditorGUILayout.EndScrollView();
        }
    }
}
#endif
