# Cell Demo: A WASM Spreadsheet in Rust with Yew Example

This is a demo for YouTube videos about Yew, topological sorting, and parsing with shunting yard algorithm.

There's also a blog post about using Tailwind with Yew [here](https://lakret.net/blog/2023-03-10-tailwind-with-yew).

## Running

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

Install Rust WASM toolchain and Trunk (WASM packaging tool):

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

Now you can run the app:

```sh
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve --port 8082 --release
```

Note, that I recommend running it in release mode, since debug mode is very slow.

## "Prod" Serving

The content of the `dist/` directory generated by `trunk build`
could be served by any web server that can handle static files.

For example:

```sh
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk build --release -d prod_dist

# e.g., serve with python:
python -m http.server 8083 --directory prod_dist
```

... and you can see the app working on localhost:8083.

## Note on Using Unstable `web-sys` APIs

To use ustable web-sys APIs such as `Clipboard`, you'll need to do the following:

1. Enable the specific web-sys crate features you're planning to use, e.g.:

```yaml
[dependencies]
web-sys = { version = "0.3.51", features = ["Clipboard"] }
```

2. Make sure to pass `RUSTFLAGS='--cfg=web_sys_unstable_apis'` when you call trunk:

```sh
RUSTFLAGS='--cfg=web_sys_unstable_apis' trunk serve --port 8082 --release
```

3. If you're using rust-analyzer, e.g. in VSCode, you should add this to your `settings.json`:

```json
"rust-analyzer.cargo.extraEnv": {
  "RUSTFLAGS": "--cfg=web_sys_unstable_apis"
}
```
