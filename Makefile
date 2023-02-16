all: linux clear windows jar

jar:
	./gradlew.sh :minecraft:build

linux_lib:
	cd ayaya_native && cargo build --profile production --target x86_64-unknown-linux-gnu

linux_move:
	cp -f ./ayaya_native/target/x86_64-unknown-linux-gnu/production/libayaya_native.so  minecraft/src/main/resources

windows_lib: 
	cd ayaya_native && cross build --profile production --target x86_64-pc-windows-gnu

windows_move:
	cp -f ./ayaya_native/target/x86_64-pc-windows-gnu/production/ayaya_native.dll minecraft/src/main/resources

windows: windows_lib windows_move

linux: linux_lib linux_move

clippy:
	cd ./ayaya_native/main/ && cargo clippy --no-default-features --features "skip_buildrs ffmpeg" -- -D warnings

clear:
	cd ayaya_native && cargo clean && cd ..

clear_jar:
	rm minecraft/src/main/resource/libayaya_native.so 2> /dev/null || echo > /dev/null
	rm minecraft/src/resource/ayaya_native.dll 2> /dev/null || echo > /dev/null
