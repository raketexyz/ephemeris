FROM docker.io/library/rust:latest as builder
WORKDIR /usr/src/ephemeris
COPY . .
RUN cargo build --release

FROM debian:buster-slim
RUN apt-get update && apt-get install -y postgresql postgresql-client && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/ephemeris/target/release/ephemeris /usr/local/bin/ephemeris
CMD ["ephemeris"]