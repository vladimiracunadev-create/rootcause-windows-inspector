#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/rootcause.ico");
    res.set("ProductName", "RootCause Windows Inspector");
    res.set(
        "FileDescription",
        "RootCause Windows Inspector - Diagnostico transparente de lentitud en Windows",
    );
    res.set("CompanyName", "Vladimir Acuña Dev");
    res.set("InternalName", "rootcause.exe");
    res.set("OriginalFilename", "rootcause.exe");
    res.set("LegalCopyright", "Copyright (c) Vladimir Acuña Dev");

    res.compile()
        .expect("No se pudo compilar el recurso Windows para RootCause Windows Inspector");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
