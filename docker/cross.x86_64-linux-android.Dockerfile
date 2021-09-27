FROM ubuntu:20.04

COPY common.sh lib.sh /
RUN /common.sh

COPY cmake.sh /
RUN /cmake.sh

COPY xargo.sh /
RUN /xargo.sh

COPY android-ndk.sh /
RUN /android-ndk.sh x86_64 23
ENV PATH=$PATH:/android-ndk/bin

COPY android-system.sh /
RUN /android-system.sh x86_64

# Using qemu allows older host cpus (without sse4) to execute the target binaries
COPY qemu.sh /
RUN /qemu.sh x86_64

RUN cp /android-ndk/sysroot/usr/lib/x86_64-linux-android/libc++_shared.so /system/lib64/
RUN cp /android-ndk/sysroot/usr/lib/x86_64-linux-android/23/libz.so /system/lib64/

# Libz is distributed in the android ndk, but for some unknown reason it is not
# found in the build process of some crates, so we explicit set the DEP_Z_ROOT
ENV CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER=x86_64-linux-android-gcc \
    CARGO_TARGET_X86_64_LINUX_ANDROID_RUNNER="qemu-x86_64 -cpu qemu64,+mmx,+sse,+sse2,+sse3,+ssse3,+sse4.1,+sse4.2,+popcnt" \
    CC_x86_64_linux_android=x86_64-linux-android-gcc \
    CXX_x86_64_linux_android=x86_64-linux-android-g++ \
    DEP_Z_INCLUDE=/android-ndk/sysroot/usr/include/ \
    RUST_TEST_THREADS=1 \
    HOME=/tmp/ \
    TMPDIR=/tmp/ \
    ANDROID_DATA=/ \
    ANDROID_DNS_MODE=local \
    ANDROID_ROOT=/system

# Install the latest version of LLVM and other dependencies
RUN ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime
RUN DEBIAN_FRONTEND=noninteractive  apt-get update \
    && apt-get install -y lsb-release software-properties-common apt-transport-https ca-certificates \
                        libclang-dev clang curl build-essential \
                        git python3-distutils python3-numpy zip unzip curl wget cmake pkg-config libtinfo5 \
    && bash -c "$(curl https://apt.llvm.org/llvm.sh)"

ARG BAZEL_VERSION=4.0.0
RUN wget -O /bazel https://github.com/bazelbuild/bazel/releases/download/${BAZEL_VERSION}/bazel-${BAZEL_VERSION}-installer-linux-x86_64.sh && \
    bash /bazel && \
    rm -f /bazel

ENV RUNECORAL_BUILD_ENVIRONMENT=true
ENV ANDROID_NDK_HOME /android/ndk
