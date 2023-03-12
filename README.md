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

# TODO

- [x] tailwind (https://tailwindcss.com/docs/installation)
- [ ] basic layout
- [ ] editable cells
- [ ] formula parser (nom?)
- [ ] evaluator
