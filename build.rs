// Executable icon on Windows (assets/brand/icon.ico, generated from icon.svg).
fn main() {
    println!("cargo:rerun-if-changed=assets/brand/icon.ico");
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/brand/icon.ico");
        res.set("ProductName", "Lienzo UI");
        res.set("FileDescription", "Lienzo UI · design interfaces for Unity");
        if let Err(e) = res.compile() {
            println!("cargo:warning=could not embed the icon: {e}");
        }
    }
}
