# Scrum poker

![Logo](readme-logo.svg)

While practicing scrum we've noticed there are nearly none free online scrum poker tools. Now there is at least one.

If you'd like to play scrum poker right away, it's available at [poker.kek.today](https://poker.kek.today/).

## Development

Not sure why would you like to do this, but if you do, here is how you can build and run this service locally:

1. Install the latest [rust toolchain](https://www.rust-lang.org/tools/install)
1. Install cargo-leptos to build the project:

    ```bash
    cargo install cargo-leptos
    ```

1. Install some npm stuff:

    ```bash
    npm install -D tailwindcss
    npm install -D daisyui@latest
    ```

    If you're using linux and can't get current npm version, you should probably take a look [here](https://github.com/nodesource/distributions) (seriously, I don't know, how TF are you supposed to find that out)

1. Build:

    ```bash
    cargo leptos build
    ```

1. Start the server:

    ```
    cargo leptos serve
    ```

1. 🪄 You are awesome

Or you could have a look at the [ci container setup](https://github.com/domwst/scrum-poker/blob/main/.build-container/Dockerfile).


### Troubleshooting

If this project doesn't build properly and there is word "nightly" somewhere in the compiler's output, this probably means that your compiler did't have a look at the `rust-toolchain.yaml`, run the following command:

```bash
rustup default nightly
```

If is says somethig about wasm or webassembly, try to run the following command:

```bash
rustup target add wasm32-unknown-unknown
```

### Tech stack

You might've noticed that this poker is blazingly fast, that's all due to a fact, that it's written purely in rust (yes, even the frontend).

Key libraries/frameworks are:

- Frontend
  - [Leptos](https://leptos.dev/)
  - [Tailwind](https://tailwindcss.com/)
  - [Daisyui](https://daisyui.com/) – because I need someone to do all the hard work for me
- Backend
  - [Tokiooo](https://tokio.rs/)
  - [Axum](https://github.com/tokio-rs/axum/)

Communication is done via websockets and leptos server functions (which in turn are plain HTTP GET/POST requests).

## Running on a remote server

### From sources

1. Compile the server in `release` mode:

    ```bash
    cargo leptos build --release
    ```

1. Copy the binary from `target/release/scrum-poker`
1. Copy static stuff from `target/site`

Your folder structure should look like this:

```
scrum-poker
site/
```

In order to run the server, you have to provide it with these environment variables:

```env
LEPTOS_OUTPUT_NAME="scrum-poker"
LEPTOS_SITE_ROOT="site"
LEPTOS_SITE_PKG_DIR="pkg"
LEPTOS_SITE_ADDR="0.0.0.0:3000"
LEPTOS_RELOAD_PORT="3001"
```

Or have a look at the container setup in the [Dockerfile](https://github.com/domwst/scrum-poker/blob/main/Dockerfile).

### From docker image

```bash
docker run -d -p 3000:3000 --name scrum-poker domwst/scrum-poker
```

## From an educational standpoint

### Websockets

There are not a lot of examples on websockets usage in axum and leptos, I think this project can somehow fill the niche. Some key places to look at:

- Axum
  - [Registration of a handler](https://github.com/domwst/scrum-poker/blob/be6fc129477974fe6e949a534268344a258d52b5/src/main.rs#L94)
  - [The handler itself](https://github.com/domwst/scrum-poker/blob/be6fc129477974fe6e949a534268344a258d52b5/src/components/poker/room/backend.rs#L119)
- Leptos
  - [Manual conversion](https://github.com/domwst/scrum-poker/blob/be6fc129477974fe6e949a534268344a258d52b5/src/components/poker/room/frontend.rs#L15) from websocket stream to a signal update
  - [Conversion](https://www.youtube.com/watch?v=dQw4w9WgXcQ) of the websocket stream to a signal using [create_signal_from_stream](https://docs.rs/leptos/latest/leptos/fn.create_signal_from_stream.html) (TODO)

  ### Leptos + Axum

  If you want to learn how to make leptos and axum play together, please reffer to [official template](https://github.com/leptos-rs/start-axum) from which this project was derived.

  ### Project structure and development patterns

  Please do not learn these concepts from this project, at the moment it's poorly structured and probably implements some of the components not in a way they should be implemented. It's good enough to get the job done but I don't beleive one should learn how to structure web-applications from this repository.

  Peace ❤️
