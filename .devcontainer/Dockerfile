FROM rust:latest

RUN rustup target add thumbv6m-none-eabi
RUN rustup component add rustfmt
RUN rustup component add clippy
RUN rustup component add llvm-tools-preview
RUN cargo install cargo-binutils
RUN cargo install cargo-generate
RUN cargo install cargo-make
RUN cargo install flip-link

