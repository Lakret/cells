# Mini-Excel Rust Example

## Running

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
trunk serve --port 8082
```

Make sure that [Tailwind CLI](https://tailwindcss.com/docs/installation) is installed and is in the `PATH`.
I recommend using a standalone CLI version:

```sh
# macOS:
# curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-macos-arm64
# linux:
curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-x64
chmod +x tailwindcss-linux-x64
mv tailwindcss-linux-x64 tailwindcss
```

## "Prod" Serving

The content of the dist/ directory generated by `trunk build`
could be served by any web server that can handle static files.
For example:

```sh
trunk build --release -d prod_dist
python -m http.server 8083 --directory prod_dist
```

... and you can see the app working on localhost:8083.

# TODO

- [x] tailwind (https://tailwindcss.com/docs/installation)
- [x] basic layout
- [x] editable cells
- [ ] big input / cell sync
- [ ] formula parser (nom?)
- [ ] evaluator
