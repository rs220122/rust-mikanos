run: bootloader_build kernel_build
	./mikanos-build/devenv/run_qemu.sh ./bootloader/target/x86_64-unknown-uefi/release/bootloader.efi ./kernel/kernel.elf

bootloader_build: 
	cd ./bootloader && cargo build --release 
	cd ..
	
kernel_build:
	cd ./kernel && cargo build --release 
	cd ..