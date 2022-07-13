cmake_minimum_required(VERSION 3.22.2)
project(wolnyjni)
set(CMAKE_BUILD_TYPE Release)
find_package(JNI REQUIRED)
include_directories(${JNI_INCLUDE_DIRS})
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11")
set(SOURCE_FILES src/library.c src/colorlib.c src/colorlib.h)
add_library(wolnyjni SHARED ${SOURCE_FILES})

find_package(PkgConfig REQUIRED)
pkg_check_modules(AVCODEC  REQUIRED IMPORTED_TARGET libavcodec)
pkg_check_modules(AVUTIL  REQUIRED IMPORTED_TARGET libavutil)
pkg_check_modules(AVFORMAT REQUIRED IMPORTED_TARGET libavformat)
pkg_check_modules(SWSCALE REQUIRED IMPORTED_TARGET libswscale)
include_directories(include ${CONFIGPP_INCLUDE_DIRS} ${PQ_INCLUDE_DIRS} ${PQXX_INCLUDE_DIRS})

target_link_libraries(wolnyjni PUBLIC
        PkgConfig::AVCODEC
        PkgConfig::AVFORMAT
        PkgConfig::SWSCALE
        PkgConfig::AVUTIL
        )