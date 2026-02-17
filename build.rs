fn main() {
    println!("cargo:rerun-if-changed=entry.S");

    cc::Build::new()
        .file("entry.S")
        .compiler("aarch64-linux-gnu-gcc")
        .flag("-c")
        .compile("entry");
}
