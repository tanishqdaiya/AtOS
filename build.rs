fn main() {
    println!("cargo:rerun-if-changed=entry.S");
    println!("cargo:rerun-if-changed=src/kernel/exceptions.s");

    cc::Build::new()
        .file("entry.S")
        .file("src/kernel/exceptions.s")
        .compiler("aarch64-linux-gnu-gcc")
        .flag("-c")
        .compile("entry");
}
