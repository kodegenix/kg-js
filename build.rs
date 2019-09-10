use std::path::PathBuf;

const DUKTAPE_SRC: &str = "lib/duktape";

fn main() {
    let mut files: Vec<PathBuf> = std::fs::read_dir(DUKTAPE_SRC)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map_or(false, |s| s == "c" || s == "h"))
        .collect();

    files.sort();

    println!("cargo:rerun-if-changed={}", DUKTAPE_SRC);
    for p in files.iter() {
        println!("cargo:rerun-if-changed={}", p.display());
    }

    cc::Build::new()
        .files(files
            .into_iter()
            .filter(|p| p.extension().map_or(false, |s| s == "c")))
        .flag_if_supported("-Wimplicit-fallthrough=2")
        .compile("libduktape.a");
}
