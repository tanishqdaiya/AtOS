# AtOS
A basic Operating System created in rust, for the Raspberry Pi 3+ Hardware

# Building

```
cargo clean
cargo build --release
```

Then flattening it to the image,

```
aarch64-linux-gnu-objcopy \
target/aarch64-unknown-none/release/at-os \
-O binary kernel8.img
```