FROM rust:slim AS builder

WORKDIR /app

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /app/target/release/miq-tmp .

CMD ["./miq-tmp"]