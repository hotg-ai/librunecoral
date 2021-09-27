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

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="/linux-runner x86_64"

# Install the latest version of LLVM and other dependencies
RUN DEBIAN_FRONTEND=noninteractive apt-get update \
    && apt-get install -y lsb-release software-properties-common apt-transport-https ca-certificates \
                        libclang-dev clang curl build-essential \
                        git python3-numpy zip unzip curl wget cmake pkg-config \
                        libusb-1.0-0-dev zlib1g-dev libegl1-mesa-dev libgles2-mesa-dev \
    && bash -c "$(curl https://apt.llvm.org/llvm.sh)"

RUN add-apt-repository ppa:ubuntu-toolchain-r/test\
    && DEBIAN_FRONTEND=noninteractive apt-get install -y gcc-9 g++-9

ARG BAZEL_VERSION=4.0.0
RUN wget -O /bazel https://github.com/bazelbuild/bazel/releases/download/${BAZEL_VERSION}/bazel-${BAZEL_VERSION}-installer-linux-x86_64.sh && \
    bash /bazel && \
    rm -f /bazel

ENV RUNECORAL_BUILD_ENVIRONMENT=true
