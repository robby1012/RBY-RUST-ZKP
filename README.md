**This README is still under construction**

# ZKP Custom Chaum-Pedersen Protocol using Rust

I'm recently learn about this Chaum-Pedersen Protocol for Zero Knowledges Proof Cryptography

So decided to implement it using Rust

## gRPC guide

You can use Visual Code gRPC Clicker extension and create from proto file

## Build & Usage

You need two terminal

Terminal #1

```
cargo build --release --bin server && cd target\release && ./server

```

Terminal #2
```
cargo build --release --bin client && cd target\release && ./client

```

If you don't have rust installed can use docker.

Usage:

```
docker-compose up -d --build
```

on docker dashboard, click container, exec tab, run this command

```
/bin/bash

cd /zkp-server/target/release

./client
```

Follow the prompt.


alternative:

Use gRPC Cliker Extension to register, authenticate & verify

set to localhost:500051 and use proto file from folder proto


## on progress
- Store private key to KMS (currently embedded in library)
- Saving session ID to DB (redis or mongo)
