prepare_windows_ffmpeg:
	$(eval DIR=$(shell echo /ffmpeg))

	@if [ ! -d $(DIR) ]; then \
		echo "Dir DOES NOT exist"; \
		mkdir $(DIR); \
		curl -L https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n5.1-latest-win64-lgpl-shared-5.1.zip --output $(DIR)/ffmpeg-archive.zip; \
		unzip -qq $(DIR)/ffmpeg-archive.zip -d $(DIR); \
	fi
