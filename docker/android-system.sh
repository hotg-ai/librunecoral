#!/usr/bin/env bash

set -x
set -euo pipefail

main() {
    local arch="${1}"
    local td
    td="$(mktemp -d)"
    pushd "${td}"

    local dependencies=(
        ca-certificates
        curl
        gcc-multilib
        git
        g++-multilib
        make
        openssh-client
        patch
        python
        python3
        libncurses5
    )

    # fake java and javac, it is not necessary for what we build, but the build
    # script ask for it
    cat << EOF > /usr/bin/java
#!/usr/bin/env bash
echo "java version \"1.7.0\""
echo "OpenJDK Runtime Environment (IcedTea 2.6.9)"
echo "OpenJDK 64-Bit Server VM (build 24.131-b00, mixed mode)"
EOF

    cat << EOF > /usr/bin/javac
#!/usr/bin/env bash
echo "javac 1.7.0"
EOF

    chmod +x /usr/bin/java
    chmod +x /usr/bin/javac

    # more faking
    export ANDROID_JAVA_HOME=/tmp
    mkdir /tmp/lib/
    touch /tmp/lib/tools.jar

    apt-get update
    local purge_list=(default-jre)
    for dep in "${dependencies[@]}"; do
        if ! dpkg -L "${dep}"; then
            apt-get install --assume-yes --no-install-recommends "${dep}"
            purge_list+=( "${dep}" )
        fi
    done

    curl --retry 3 -sSfL https://storage.googleapis.com/git-repo-downloads/repo -O
    chmod +x repo

    # this is the minimum set of modules that are need to build bionic
    # this was created by trial and error
    python3 ./repo init --depth=1 -u https://android.googlesource.com/platform/manifest -b android-6.0.0_r1
    ./repo sync -c bionic
    ./repo sync -c build
    ./repo sync -c external/boringssl
    ./repo sync -c external/compiler-rt
    ./repo sync -c external/elfutils
    ./repo sync -c external/gtest
    ./repo sync -c external/jemalloc
    ./repo sync -c external/libcxx
    ./repo sync -c external/libcxxabi
    ./repo sync -c external/libselinux
    ./repo sync -c external/mksh
    ./repo sync -c external/pcre
    ./repo sync -c external/zlib
    ./repo sync -c libnativehelper
    ./repo sync -c prebuilts/misc
    ./repo sync -c prebuilts/clang/linux-x86/host/3.6
    ./repo sync -c prebuilts/gcc/linux-x86/host/x86_64-linux-glibc2.15-4.8
    ./repo sync -c system/core

    case "${arch}" in
        arm)
            python3 ./repo sync prebuilts/gcc/linux-x86/arm/arm-linux-androideabi-4.9
        ;;
        arm64)
            python3 ./repo sync prebuilts/gcc/linux-x86/arm/arm-linux-androideabi-4.9
            python3 ./repo sync prebuilts/gcc/linux-x86/aarch64/aarch64-linux-android-4.9
        ;;
        x86)
            python3 ./repo sync prebuilts/gcc/linux-x86/x86/x86_64-linux-android-4.9
        ;;
        x86_64)
            python3 ./repo sync prebuilts/gcc/linux-x86/x86/x86_64-linux-android-4.9
        ;;
    esac

    # avoid build tests
    rm bionic/linker/tests/Android.mk bionic/tests/Android.mk bionic/benchmarks/Android.mk

    set +u
    # shellcheck disable=SC1091
    source build/envsetup.sh
    lunch "aosp_${arch}-user"
    mmma bionic/
    mmma external/mksh/
    mmma system/core/toolbox/
    set -u

    if [[ "${arch}" = "arm" ]]; then
        mv out/target/product/generic/system/ /
    else
        mv "out/target/product/generic_${arch}/system"/ /
    fi

    # list from https://elinux.org/Android_toolbox
    for tool in cat chmod chown cmp cp ctrlaltdel date dmesg getprop hd id \
        ifconfig insmod kill ln lsmod lsusb md5 mkdir mv netstat notify printenv \
        reboot rm rmdir rmmod route schedtop setconsole setprop sleep smd \
        sync touch umount vmstat wipe; do
        ln -s /system/bin/toolbox "/system/bin/${tool}"
    done

    echo "127.0.0.1 localhost" > /system/etc/hosts

    if (( ${#purge_list[@]} )); then
      apt-get purge --auto-remove -y "${purge_list[@]}"
    fi

    popd

    rm -rf "${td}"
    rm "${0}"
}

main "${@}"
