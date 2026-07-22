##: Build stage
FROM rust:1-trixie AS builder

USER root

RUN apt-get update && apt-get install -y apt-utils sqlite3 openssl capnproto libzmq3-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/podping.cloud
COPY podping/ podping/
WORKDIR /opt/podping.cloud/podping
RUN cargo build --release --locked
RUN cp target/release/podping .


##: Bundle stage
# Must be the same Debian release as the builder image: the binary links
# against the builder's glibc, and a newer builder glibc than the runner's
# fails at startup with "version `GLIBC_X.YZ' not found".
FROM debian:trixie-slim AS runner

USER root

RUN apt-get update && apt-get install -y apt-utils sqlite3 openssl capnproto libzmq3-dev libc6 && rm -rf /var/lib/apt/lists/*

RUN chown -R 1000:1000 /opt
RUN mkdir /data && chown -R 1000:1000 /data

USER 1000
RUN mkdir /opt/podping

WORKDIR /opt/podping
COPY --from=builder /opt/podping.cloud/podping/target/release/podping .
COPY --from=builder /opt/podping.cloud/podping/home.html .

EXPOSE 80/tcp

ENTRYPOINT ["/opt/podping/podping"]
