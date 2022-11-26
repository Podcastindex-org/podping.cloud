##: Build stage
FROM rust:latest AS builder

USER root

RUN apt-get update
RUN apt-get upgrade -y
RUN apt-get install -y apt-utils
RUN apt-get install -y sqlite3
RUN apt-get install -y openssl
RUN apt-get install -y capnproto
RUN apt-get install -y libzmq3-dev

WORKDIR /opt
ARG GIT_REPO=https://github.com/Podcastindex-org/podping.cloud.git
ARG GIT_BRANCH=main
RUN git clone -b ${GIT_BRANCH} ${GIT_REPO}
WORKDIR /opt/podping.cloud/podping
RUN cargo build --release
RUN cp target/release/podping .


##: Bundle stage
FROM debian:bullseye-slim AS runner

USER root

RUN apt-get update
RUN apt-get upgrade -y
RUN apt-get install -y apt-utils
RUN apt-get install -y sqlite3
RUN apt-get install -y openssl
RUN apt-get install -y capnproto
RUN apt-get install -y libzmq3-dev

RUN chown -R 1000:1000 /opt
RUN mkdir /data && chown -R 1000:1000 /data

USER 1000
RUN mkdir /opt/podping

WORKDIR /opt/podping
COPY --from=builder /opt/podping.cloud/podping/target/release/podping .
COPY --from=builder /opt/podping.cloud/podping/home.html .

EXPOSE 80/tcp

ENTRYPOINT ["/opt/podping/podping"]