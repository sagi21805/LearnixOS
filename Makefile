# Tools
OBJCPY = objcopy
CARGO = cargo +nightly
QEMU = qemu-system-x86_64
OUTPUT = kernel.img

# Stages
FIRST = stages/first_stage
SECOND = stages/second_stage
KERNEL = kernel

# Artifact Path
16BIT_ARTIFACT = target/16bit_target/release/first_stage
32BIT_ARTIFACT = target/32bit_target/release/second_stage
64BIT_ARTIFACT = target/64bit_target/release/kernel

# Targets
TARGET_DIR = build/targets
16BIT_TARGET = $(TARGET_DIR)/16bit_target
32BIT_TARGET = $(TARGET_DIR)/32bit_target
64BIT_TARGET = $(TARGET_DIR)/64bit_target

# Manifests
TOML = Cargo.toml

# Default rule
all: build-rust create_img

# Step 1: Build Rust code
build-rust:
	@echo "Compiling Rust code..."
	$(CARGO) build --release --manifest-path $(FIRST)/$(TOML)  --target $(16BIT_TARGET).json 
	$(CARGO) build --release --manifest-path $(SECOND)/$(TOML) --target $(32BIT_TARGET).json 
	$(CARGO) build --release --manifest-path $(KERNEL)/$(TOML) --target $(64BIT_TARGET).json
# Step 3: Link everything
create_img: build-rust build-asm
	$(OBJCPY) -I elf32-i386 -O binary $(16BIT_ARTIFACT) 16bit.img                    
	$(OBJCPY) -I elf32-i386 -O binary $(32BIT_ARTIFACT) 32bit.img
	$(OBJCPY) -I elf64-x86-64 -O binary $(64BIT_ARTIFACT) 64bit.img
	cat 16bit.img 32bit.img 64bit.img > $(OUTPUT)

# Step 4: Run in QEMU
run: all
	@echo "Running bootloader in QEMU..."
	$(QEMU) -m 4096 -smp 1 -drive format=raw,file=$(OUTPUT)


# Clean build artifacts
clean:
	@echo "Cleaning build files..."
	$(CARGO) clean 
	rm -rf *.img 

.PHONY: all build-rust build-asm link run clean
