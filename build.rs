// Icono del ejecutable en Windows (assets/marca/icono.ico, generado desde icono.svg).
fn main() {
    println!("cargo:rerun-if-changed=assets/marca/icono.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/marca/icono.ico");
        res.set("ProductName", "Lienzo UI");
        res.set("FileDescription", "Lienzo UI · diseña interfaces para Unity");
        if let Err(e) = res.compile() {
            println!("cargo:warning=no se pudo incrustar el icono: {e}");
        }
    }
}
