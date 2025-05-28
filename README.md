**This README is still under construction**

# ZKP Custom Chaum-Pedersen Protocol using Rust

I'm recently learn about this Chaum-Pedersen Protocol for Zero Knowledges Proof Cryptography

So decided to implement it using Rust

## gRPC guide

You can use Visual Code gRPC Clicker extension to register the user

or you can set the docker-compose.yaml ENVIRONMENT variable for testing.

example:

```
environment:
    - USER=RBYSTNL
```

## Server & Client Binaries Usage

### Docker

You can run the program with Docker. First build the containers:

```
$ docker-compose build zkpserver
```

Run the container:

```
$ docker-compose run --rm zkpserver
```

In the remote terminal that appears run the server:

```
root@e84736012f9a:/zkp-server# cargo run --bin server --release
```

Open a new terminal on your machine and connect to the container:

```
$ docker container ls
CONTAINER ID   IMAGE                  COMMAND   CREATED          STATUS          PORTS     NAMES
e84736012f9a   zkp-course-zkpserver   "bash"    20 minutes ago   Up 20 minutes             zkp-course_zkpserver_run_b1f3fa2cd94a

$ docker exec -it e84736012f9a /bin/bash
```

Run the client:

```
root@e84736012f9a:/zkp-server# cargo run --bin client --release
```

