# Tools
OBJCPY = objcopy
CARGO = cargo +nightly
QEMU = qemu-system-x86_64
OUTPUT = bootloader.img

# Stages
FIRST = first_stage
SECOND = second_stage
KERNEL = kernel

# Targets
16BIT = 16bit_target
32BIT = 32bit_target
64BIT = 64bit_target

# Manifests
TOML = Cargo.toml

# Default rule
all: build-rust create_img

# Step 1: Build Rust code
build-rust:
	@echo "Compiling Rust code..."
	$(CARGO) build --release --manifest-path ./$(FIRST)/$(TOML) --target ./$(FIRST)/$(16BIT).json 
	$(CARGO) build --release --manifest-path ./$(SECOND)/$(TOML) --target ./$(SECOND)/$(32BIT).json 
	$(CARGO) build --release --manifest-path ./$(KERNEL)/$(TOML) --target ./$(KERNEL)/$(64BIT).json
# Step 3: Link everything
create_img: build-rust build-asm
	$(OBJCPY) -I elf32-i386 -O binary target/$(16BIT)/release/$(FIRST) 16bit.img                    
	$(OBJCPY) -I elf32-i386 -O binary target/$(32BIT)/release/$(SECOND) 32bit.img
	$(OBJCPY) -I elf64-x86-64 -O binary target/$(64BIT)/release/$(KERNEL) kernel.img
	cat 16bit.img 32bit.img kernel.img> $(OUTPUT)

# Step 4: Run in QEMU
run: all
	@echo "Running bootloader in QEMU..."
	$(QEMU) -drive format=raw,file=$(OUTPUT)

# Clean build artifacts
clean:
	@echo "Cleaning build files..."
	$(CARGO) clean 
	rm -rf *.img 

.PHONY: all build-rust build-asm link run clean
