FROM rust:1.88.0-slim-bullseye as build

WORKDIR /build

RUN apt-get update
RUN apt-get install -y --no-install-recommends curl pkg-config libssl-dev make ca-certificates gcc g++ libc6-dev
RUN curl -fsSL https://deb.nodesource.com/setup_21.x | bash -
RUN apt-get install -y nodejs
RUN npm install -D tailwindcss
RUN npm install -D daisyui@latest
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked cargo-leptos

COPY . .

# RUN rm /build/Cargo.lock

RUN cargo leptos build --release

FROM debian:bullseye-slim as final

WORKDIR /app

COPY --from=build /build/target/release/scrum-poker scrum-poker
COPY --from=build /build/target/site site

ENV LEPTOS_OUTPUT_NAME="scrum-poker"
ENV LEPTOS_SITE_ROOT="site"
ENV LEPTOS_SITE_PKG_DIR="static"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_RELOAD_PORT="3001"

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "1234" \
    appuser

RUN chown -R appuser:appuser /app
RUN chmod -R 755 /app

USER appuser

EXPOSE 3000

CMD ["/app/scrum-poker"]
