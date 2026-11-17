// Exportador de ejemplo: recibe el diseño y devuelve el texto del archivo que se guardará.

fn celda(s) {
    let x = s;
    x.replace("\"", "\"\"");
    "\"" + x + "\""
}

fn exportar(diseno) {
    let salida = "elemento,texto\n";
    for e in diseno.elements {
        if e.text != "" {
            salida += celda(e.name) + "," + celda(e.text) + "\n";
        }
    }
    salida
}
