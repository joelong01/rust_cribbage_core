FROM rust:1.55 as build

WORKDIR /cribbage

COPY ./api ./api
COPY ./game ./game

WORKDIR /cribbage/api

RUN cargo build --release

RUN mkdir -p /build
RUN cp target/release/cribbage-api /build/

FROM ubuntu:18.04

RUN apt-get update && apt-get -y upgrade
RUN apt-get -y install openssl
RUN apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=build /build/cribbage-api /

CMD ["/cribbage-api"]
