FROM boymaas/rust-build:latest as planner
WORKDIR /source
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

# --- CACHER ---
FROM boymaas/rust-build:latest AS cacher
WORKDIR /source

COPY --from=planner /source/recipe.json recipe.json
RUN for i in {1..10}; do \
       cargo chef cook \
         --check \
         --release \
         --recipe-path recipe.json && break || sleep 25; \
    done

# --- BUILDER ---
FROM boymaas/rust-build:latest AS builder
WORKDIR /source
# Now we copy code, not to influence previous
# caching layer
COPY . .

COPY --from=cacher /source/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo

RUN cargo build --release

# --- RUNTIME ---
FROM debian:buster AS runtime
WORKDIR /app
COPY --from=builder /source/target/release/node .
CMD ["/app/node"]
