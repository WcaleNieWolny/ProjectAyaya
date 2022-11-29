compile:
	cd ayaya_native && cargo build --profile production --target x86_64-unknown-linux-gnu
	cp -f ayaya_native/target/x86_64-unknown-linux-gnu/production/libayaya_native.so  minecraft/src/main/resource
	./gradlew.sh :minecraft:build  
