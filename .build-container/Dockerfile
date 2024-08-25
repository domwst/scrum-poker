FROM rust:1.77.0-slim-bullseye as build

WORKDIR /build

RUN apt-get update
RUN apt-get install -y curl
RUN curl -fsSL https://deb.nodesource.com/setup_21.x | bash -
RUN apt-get install -y nodejs
RUN npm install -D tailwindcss
RUN npm install -D daisyui@latest
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos # Bump
