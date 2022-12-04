linux_jar:
	cd ayaya_native && cargo build --profile production --target x86_64-unknown-linux-gnu
	cp -f ayaya_native/target/x86_64-unknown-linux-gnu/production/libayaya_native.so  minecraft/src/main/resource
	./gradlew.sh :minecraft:build

prepare_windows_ffmpeg:
	$(eval DIR=$(shell echo ./ayaya_native/target/ffmpeg_win/))

	@if [ ! -d ./ayaya_native/target/ ]; then \
		mkdir ./ayaya_native/target/; \
	fi

	@if [ ! -d $(DIR) ]; then \
		echo "Dir DOES NOT exist"; \
		mkdir $(DIR); \
		curl -L https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-lgpl-shared.zip --output $(DIR)/ffmpeg-archive.zip; \
		unzip -qq $(DIR)/ffmpeg-archive.zip -d $(DIR); \
	fi

windows_lib:
	$(MAKE) prepare_windows_ffmpeg
	$(eval DIR=$(shell realpath ./ayaya_native/target/ffmpeg_win/))
	cd ayaya_native && PKG_CONFIG_SYSROOT_DIR=/ PKG_CONFIG_PATH=$(DIR)/ffmpeg-master-latest-win64-lgpl-shared/lib/pkgconfig  cargo build --profile production --target x86_64-pc-windows-gnu
