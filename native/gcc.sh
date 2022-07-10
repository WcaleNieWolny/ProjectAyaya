clang_bin=`which clang`

if [ -z $clang_bin ]; then
    clang_ver=`dpkg --get-selections | grep clang | grep -v -m1 libclang | cut -f1 | cut -d '-' -f2`
    clang_bin="clang-$clang_ver"
    clang_xx_bin="clang++-$clang_ver"
fi

buildj () {

    rm -rf build
    mkdir build
    cd build
    cmake ../ -DCMAKE_C_COMPILER=$clang_bin
    make
    ls
    cd ../
    echo 'Build finished.'
}

buildj