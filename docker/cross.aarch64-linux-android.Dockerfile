FROM ubuntu:20.04

COPY common.sh lib.sh /
RUN /common.sh

COPY cmake.sh /
RUN /cmake.sh

COPY xargo.sh /
RUN /xargo.sh

COPY android-ndk.sh /
RUN /android-ndk.sh
ENV PATH=$PATH:/android/ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/

# Create a minimal android system
COPY android-system.sh /
RUN /android-system.sh arm64 \
    && cp /android/ndk/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/libc++_shared.so /system/lib64/ \
    && cp /android/ndk/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android/23/libz.so /system/lib64/

COPY qemu.sh /
RUN /qemu.sh aarch64

# Libz is distributed in the android ndk, but for some unknown reason it is not
# found in the build process of some crates, so we explicit set the DEP_Z_ROOT
ENV CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=aarch64-linux-android23-clang \
    CARGO_TARGET_AARCH64_LINUX_ANDROID_RUNNER=qemu-aarch64 \
    CC_aarch64_linux_android=aarch64-linux-android23-clang \
    CXX_aarch64_linux_android=aarch64-linux-android23-clang++ \
    DEP_Z_INCLUDE=/android/ndk/sysroot/usr/include/ \
    RUST_TEST_THREADS=1 \
    HOME=/tmp/ \
    TMPDIR=/tmp/ \
    ANDROID_DATA=/ \
    ANDROID_DNS_MODE=local \
    ANDROID_ROOT=/system \
    ANDROID_NDK_HOME=/android/ndk \
    RUNECORAL_BUILD_ENVIRONMENT=true

# Install the latest version of LLVM and other dependencies
RUN ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime \
    && DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y lsb-release software-properties-common apt-transport-https ca-certificates \
                        libclang-dev clang build-essential \
                        git python3-distutils python3-numpy zip unzip curl wget cmake pkg-config libtinfo5 \
    && bash -c "$(curl https://apt.llvm.org/llvm.sh)" \
    && rm -rf /var/lib/apt/* \
    && rm -rf /var/cache/apt/*

ARG BAZEL_VERSION=4.0.0
RUN wget -O /bazel https://github.com/bazelbuild/bazel/releases/download/${BAZEL_VERSION}/bazel-${BAZEL_VERSION}-installer-linux-x86_64.sh && \
    bash /bazel && \
    rm -f /bazel

RUN export CARGO_HOME=/tmp/cargo && export RUSTUP_HOME=/tmp/rustup && HOME=/tmp/ \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup-init.sh && \
    sh rustup-init.sh --default-toolchain stable -y && \
    . $CARGO_HOME/env && \
    rustup update && \
    cargo install bindgen && \
    mv $CARGO_HOME/bin/bindgen /usr/bin && \
    rm -r "${RUSTUP_HOME}" "${CARGO_HOME}" && \
    unset CARGO_HOME && unset RUSTUP_HOME
