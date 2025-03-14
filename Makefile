# Tools
OBJCPY = objcopy
CARGO = cargo +nightly
QEMU = qemu-system-x86_64
OUTPUT = bootloader.img

# Stages
FIRST = first_stage
SECOND = second_stage
TEST = test_stage

# Targets
16BIT = 16bit_target
32BIT = 32bit_target

# Manifests
TOML = Cargo.toml

# Default rule
all: build-rust create_img

# Step 1: Build Rust code
build-rust:
	@echo "Compiling Rust code..."
	$(CARGO) build --release --manifest-path ./$(FIRST)/$(TOML) --target ./$(FIRST)/$(16BIT).json -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem
	$(CARGO) build --release --manifest-path ./$(SECOND)/$(TOML) --target ./$(SECOND)/$(32BIT).json -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem
# Step 3: Link everything
create_img: build-rust build-asm
	$(OBJCPY) -I elf32-i386 -O binary target/$(16BIT)/release/$(FIRST) 16bit.img                    
	$(OBJCPY) -I elf32-i386 -O binary target/$(32BIT)/release/$(SECOND) 32bit.img
	cat 16bit.img 32bit.img > $(OUTPUT)
	rm 16bit.img 32bit.img

# Step 4: Run in QEMU
run: all
	@echo "Running bootloader in QEMU..."
	$(QEMU) -drive format=raw,file=$(OUTPUT)

# Clean build artifacts
clean:
	@echo "Cleaning build files..."
	$(CARGO) clean 
	rm -rf *.img 

test: 
	@echo "Compiling Rust code..."
	$(CARGO) build --release --manifest-path ./$(FIRST)/$(TOML) --target ./$(FIRST)/$(16BIT).json -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem
	$(CARGO) build --release --manifest-path ./$(TEST)/$(TOML) --target ./$(TEST)/$(16BIT).json -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem
	$(OBJCPY) -I elf32-i386 -O binary target/$(16BIT)/release/$(FIRST) 16bit.img                    
	$(OBJCPY) -I elf32-i386 -O binary target/$(16BIT)/release/$(TEST) test.img
	cat 16bit.img test.img > $(OUTPUT)
	rm 16bit.img test.img
	$(QEMU) -drive format=raw,file=$(OUTPUT)

.PHONY: all build-rust build-asm link run clean
