FROM ubuntu:bionic

COPY linux-image.sh /
RUN /linux-image.sh x86_64

COPY common.sh lib.sh /
RUN /common.sh

COPY cmake.sh /
RUN /cmake.sh

COPY xargo.sh /
RUN /xargo.sh

COPY qemu.sh /
RUN /qemu.sh x86_64 softmmu

COPY dropbear.sh /
RUN /dropbear.sh

COPY linux-runner /

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="/linux-runner x86_64" \
    RUNECORAL_BUILD_ENVIRONMENT=true

# Install the latest version of LLVM and other dependencies
RUN ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime \
    && DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y lsb-release software-properties-common apt-transport-https ca-certificates \
                        libclang-dev clang build-essential \
                        git python3-distutils python3-numpy zip unzip curl wget cmake pkg-config libtinfo5 \
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
