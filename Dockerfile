FROM boymaas/rust-build:latest AS builder
WORKDIR /source

COPY . .

RUN for i in {1..10}; do \
      cargo build --release && break || sleep 15; \
    done

# --- RUNTIME ---
FROM debian:buster AS runtime
WORKDIR /app
COPY --from=builder /source/target/release/node .
CMD ["/app/node"]
