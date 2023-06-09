# Bevy + GGRS

Shows how to use `matchbox_socket` with `bevy` and `ggrs` using `bevy_matchbox` and `bevy_ggrs`, to create a simple working browser "game" (if moving cubes around on a plane can be called a game).

## Live Demo

There is a live version here (move the cube with WASD):

- 2-Player: <https://gamedevalice.github.io/box_game/>
- 3-Player: <https://gamedevalice.github.io/box_game/?players=3>
- N-player: Edit the link above.

When enough players have joined, you should see a couple of boxes, one of which
you can move around using the `WASD` keys.

You can open the browser console to get some rough idea about what's happening
(or not happening if that's the unfortunate case).

## Instructions

- Run the matchbox-provided [`matchbox_server`](https://github.com/johanhelsing/matchbox/tree/main/matchbox_server), or run your own on `ws://localhost:3536/`.
- Run the demo (enough clients must connect before the game stats)
  - [on Native](#run-on-native)
  - [on WASM](#run-on-wasm)

## Run on Native

```sh
cargo run -- [--matchbox ws://127.0.0.1:3536] [--players 2] [--room <name>]
```

## Run on WASM

### Prerequisites

Install the `wasm32-unknown-unknown` target

```sh
rustup target install wasm32-unknown-unknown
```

Install a lightweight web server

```sh
cargo install wasm-server-runner
```

### Serve

```sh
cargo run --target wasm32-unknown-unknown
```

### Run

- Use a web browser and navigate to <http://127.0.0.1:1334/?players=2>
- Open the console to see execution logs

## Build for Github pages

### Setup

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo install wasm-opt
cargo install basic-http-server
```

### Build

```sh
cargo build --profile wasm-release --target wasm32-unknown-unknown
wasm-bindgen --out-name app --out-dir docs/wasm --target web target/wasm32-unknown-unknown/wasm-release/box_game.wasm
wasm-opt -Oz --output docs/wasm/app_bg.wasm docs/wasm/app_bg.wasm
cp -r -Force assets/ docs/
```

### Test

```
basic-http-server -a 0.0.0.0:4000
```

### Running matchbox_server Dockerfile

```sh
cd matchbox_server
docker build . -t matchbox_server:latest
docker images
docker run -it -p 3536:3536 matchbox_server:latest
docker tag matchbox_server:latest registry.digitalocean.com/matchbox-server/matchbox_server:v1.0.3
docker push registry.digitalocean.com/matchbox-server/matchbox_server:v1.0.3
```
