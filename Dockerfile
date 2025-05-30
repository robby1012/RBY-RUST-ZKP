FROM rust:1.70

WORKDIR /zkp-server

COPY . .

RUN apt update
RUN apt install -y protobuf-compiler

RUN cargo build --release --bin server
RUN cp target/release/server /usr/local/bin/zkp-server
CMD ["zkp-server"]
