FROM ghcr.io/cross-rs/x86_64-pc-windows-gnu:latest

RUN apt-get -o Acquire::Check-Valid-Until=false -o Acquire::Check-Date=false update

RUN apt-get install --assume-yes unzip

COPY Makefile /

RUN make prepare_windows_ffmpeg

ENV PKG_CONFIG_PATH=/ffmpeg/ffmpeg-n5.1-latest-win64-lgpl-shared-5.1/lib/pkgconfig
