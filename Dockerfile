# syntax=docker/dockerfile:1.3-labs
FROM rust:1.55 as build

# capture dependencies
COPY Cargo.toml Cargo.lock /app/

RUN cargo new --lib /app/game
COPY game/Cargo.toml /app/game/

RUN cargo new /app/api
COPY api/Cargo.toml /app/api/

WORKDIR /app/api
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo build --release

# build the app
COPY ./api /app/api
COPY ./game /app/game
RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  # update timestamps to force a new build
  touch /app/game/src/lib.rs /app/api/src/main.rs
  cargo build --release
EOF

CMD ["/app/target/release/cribbage-api"]

# slim runtime image
FROM debian:buster-slim as app
COPY --from=build /app/target/release/cribbage-api /cribbage-api
CMD ["/cribbage-api"]
