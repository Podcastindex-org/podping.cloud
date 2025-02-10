##: Build stage
FROM rust:latest AS builder

USER root

RUN apt update && apt install -y apt-utils sqlite3 openssl capnproto libzmq3-dev && rm -rf /var/lib/apt/lists/*

COPY ./podping/ /opt/podping.cloud/podping

WORKDIR /opt/podping.cloud/podping
RUN cargo build --release
RUN cp target/release/podping .


##: Bundle stage
FROM debian:bookworm-slim AS runner

USER root

RUN apt update && apt install -y sqlite3 openssl && rm -rf /var/lib/apt/lists/*

RUN chown -R 1000:1000 /opt
RUN mkdir /data && chown -R 1000:1000 /data

USER 1000
RUN mkdir /opt/podping

WORKDIR /opt/podping
COPY --from=builder /opt/podping.cloud/podping/target/release/podping .
COPY --from=builder /opt/podping.cloud/podping/home.html .

EXPOSE 80/tcp

ENTRYPOINT ["/opt/podping/podping"]