run: build 
	./mikanos-build/devenv/run_qemu.sh ./bootloader/target/x86_64-unknown-uefi/release/bootloader.efi

build: 
	cd ./bootloader && cargo build --release 
	cd ..
	