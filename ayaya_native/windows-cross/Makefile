prepare_windows_ffmpeg:
	$(eval DIR=$(shell echo /ffmpeg))

	@if [ ! -d $(DIR) ]; then \
		echo "Dir DOES NOT exist"; \
		mkdir $(DIR); \
		curl -L https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-lgpl-shared.zip --output $(DIR)/ffmpeg-archive.zip; \
		unzip -qq $(DIR)/ffmpeg-archive.zip -d $(DIR); \
	fi
