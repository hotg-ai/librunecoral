FROM ubuntu:bionic

COPY common.sh lib.sh /
RUN /common.sh

COPY cmake.sh /
RUN /cmake.sh

COPY xargo.sh /
RUN /xargo.sh

RUN apt-get install --assume-yes --no-install-recommends \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross

COPY qemu.sh /
RUN /qemu.sh aarch64 softmmu

COPY dropbear.sh /
RUN /dropbear.sh

COPY linux-image.sh /
RUN /linux-image.sh aarch64

COPY linux-runner /

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUNNER="/linux-runner aarch64" \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++ \
    QEMU_LD_PREFIX=/usr/aarch64-linux-gnu \
    RUST_TEST_THREADS=1 \
    RUNECORAL_BUILD_ENVIRONMENT=true

# Install the latest version of LLVM and other dependencies
RUN ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime \
    && dpkg --add-architecture arm64 && apt-get update \
    && apt-get install -y lsb-release software-properties-common apt-transport-https ca-certificates \
                        libclang-dev clang curl build-essential \
                        crossbuild-essential-arm64 \
                        git python3-numpy zip unzip curl wget cmake pkg-config \
                        libusb-1.0-0-dev:arm64 zlib1g-dev:arm64 libegl1-mesa-dev:arm64 libgles2-mesa-dev:arm64 \
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
