FROM boymaas/rust-build:latest AS builder
WORKDIR /source

COPY . .

RUN cargo build --release

# --- RUNTIME ---
FROM debian:buster AS runtime
WORKDIR /app
COPY --from=builder /source/target/release/node .
CMD ["/app/node"]
