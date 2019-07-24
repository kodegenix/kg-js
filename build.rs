use std::path::PathBuf;

fn main() {
    let mut files: Vec<PathBuf> = std::fs::read_dir("lib/duktape")
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| match p.extension() {
            Some(s) => s == "c",
            None => false
        })
        .collect();

    files.sort();

    cc::Build::new()
        .files(files)
        .flag_if_supported("-Wimplicit-fallthrough=2")
        .compile("libduktape.a");
}
