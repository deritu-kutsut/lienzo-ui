// Comando de ejemplo: recibe el diseño (mapa con title, width, height, elements…) y lo devuelve modificado.
// Cada elemento tiene x, y, w, h (píxeles del lienzo), name, kind, text, fill [r,g,b,a]…

fn redondear(v, paso) {
    (v / paso).round() * paso
}

fn ejecutar(diseno) {
    let paso = 8.0;
    let cambiados = 0;
    for i in 0..diseno.elements.len() {
        let e = diseno.elements[i];
        let antes = [e.x, e.y, e.w, e.h];
        e.x = redondear(e.x, paso);
        e.y = redondear(e.y, paso);
        e.w = redondear(e.w, paso);
        e.h = redondear(e.h, paso);
        if e.w < paso { e.w = paso; }
        if e.h < paso { e.h = paso; }
        if antes != [e.x, e.y, e.w, e.h] { cambiados += 1; }
        diseno.elements[i] = e;
    }
    log(`${cambiados} elementos ajustados a ${paso} px`);
    diseno
}
