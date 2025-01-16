FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
apt-get install -y pkg-config libssl-dev libclang-dev curl wget git build-essential unzip gcc-mingw-w64&& \
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN git clone https://github.com/Business1sg00d/HTTPShell /opt/HTTPShell && \
rustup target add x86_64-pc-windows-gnu && \
cd /opt/HTTPShell/ServiceBinary/ && cargo build --target x86_64-pc-windows-gnu && \
cd /opt/HTTPShell/InstallService/ && cargo build --target x86_64-pc-windows-gnu
