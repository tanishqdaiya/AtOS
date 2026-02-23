# I'm gonna be honest i asked chatgpt to generate this idk how this works
TARGET = aarch64-unknown-none
KERNEL = at-os
BUILD = target/$(TARGET)/release/$(KERNEL)

OBJCOPY = aarch64-linux-gnu-objcopy

QEMU = qemu-system-aarch64

# Default target
all: kernel8.img

# Build release
build:
	cargo build --release --target $(TARGET)

# Convert ELF to raw binary
kernel8.img: build
	$(OBJCOPY) $(BUILD) -O binary kernel8.img

# Clean everything
clean:
	cargo clean
	rm -f kernel8.img