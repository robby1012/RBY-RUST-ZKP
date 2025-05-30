FROM rust:1.70

WORKDIR /zkp-server

COPY . .

RUN apt update
RUN apt install -y protobuf-compiler

RUN cargo build --release --bin server
RUN cd target/release
RUN ./server
