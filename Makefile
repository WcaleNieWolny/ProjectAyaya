linux_jar:
	cd ayaya_native && cargo build --profile production --target x86_64-unknown-linux-gnu
	cp -f ayaya_native/target/x86_64-unknown-linux-gnu/production/libayaya_native.so  minecraft/src/main/resource
	./gradlew.sh :minecraft:build

windows:
	cd ayaya_native && PKG_CONFIG_SYSROOT_DIR=/ PKG_CONFIG_PATH=TODO  cargo build --profile production --target x86_64-pc-windows-gnu
