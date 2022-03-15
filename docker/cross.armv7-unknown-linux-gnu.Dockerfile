FROM ubuntu:bionic

COPY common.sh lib.sh /
RUN /common.sh

COPY cmake.sh /
RUN /cmake.sh

COPY xargo.sh /
RUN /xargo.sh

RUN apt-get install --assume-yes --no-install-recommends \
    g++-arm-linux-gnueabihf \
    libc6-dev-armhf-cross

COPY qemu.sh /
RUN /qemu.sh arm softmmu

COPY dropbear.sh /
RUN /dropbear.sh

COPY linux-image.sh /
RUN /linux-image.sh armv7

COPY linux-runner /

ENV CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUNNER="/linux-runner armv7" \
    CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc \
    CXX_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-g++ \
    QEMU_LD_PREFIX=/usr/arm-linux-gnueabihf \
    RUST_TEST_THREADS=1 \
    RUNECORAL_BUILD_ENVIRONMENT=true

# Install the latest version of LLVM and other dependencies
RUN ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime \
    && dpkg --add-architecture armhf && apt-get update \
    && apt-get install -y lsb-release software-properties-common apt-transport-https ca-certificates \
                        libclang-dev clang curl build-essential \
                        crossbuild-essential-armhf \
                        git python3-numpy zip unzip curl wget cmake pkg-config \
                        libusb-1.0-0-dev:armhf zlib1g-dev:armhf libegl1-mesa-dev:armhf libgles2-mesa-dev:armhf \
    && add-apt-repository ppa:ubuntu-toolchain-r/test \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y gcc-9 g++-9 \
    && bash -c "$(curl https://apt.llvm.org/llvm.sh)" \
    && rm -rf /var/lib/apt/* \
    && rm -rf /var/cache/apt/*

# Install Bindgen
RUN export CARGO_HOME=/tmp/cargo && export RUSTUP_HOME=/tmp/rustup && HOME=/tmp/ \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup-init.sh && \
    sh rustup-init.sh --default-toolchain stable -y && \
    . $CARGO_HOME/env && \
    rustup update && \
    cargo install bindgen && \
    mv $CARGO_HOME/bin/bindgen /usr/bin && \
    rm -r "${RUSTUP_HOME}" "${CARGO_HOME}" && \
    unset CARGO_HOME && unset RUSTUP_HOME

ARG BAZEL_VERSION=4.0.0
RUN wget -O /bazel https://github.com/bazelbuild/bazel/releases/download/${BAZEL_VERSION}/bazel-${BAZEL_VERSION}-installer-linux-x86_64.sh && \
    bash /bazel && \
    rm -f /bazel
