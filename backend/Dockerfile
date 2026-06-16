FROM rust:1.85-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY src src

RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/wastewater-sim /app/wastewater-sim

EXPOSE 8080

CMD ["./wastewater-sim"]
