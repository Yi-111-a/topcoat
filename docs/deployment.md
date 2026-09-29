# Deployment

There is no `topcoat serve` command. A Topcoat app is an ordinary Rust binary, so deployment is
`cargo build`, `topcoat asset bundle`, and a process supervisor. This guide covers the points where
the build and the runtime have to agree, then the hosting decisions behind a reverse proxy.

## Build the binary and bundle it with the same profile

Build with the `release` profile and bundle with that profile too:

```sh
cargo build --release
topcoat asset bundle --release
```

`topcoat asset bundle` takes the same profile flags as `cargo build` and builds the app itself to
scan the executable. It defaults to the `dev` profile, so a bare `topcoat asset bundle` produces the
bundle for the debug binary and leaves the release binary without one. Each profile keeps its own
`assets` directory next to its executable:

```text
target/release/my-app
target/release/assets/manifest.toml
```

Two things travel to the server: the executable, and the `assets` directory beside it.
`AssetBundle::load` only looks next to the running executable, so the two must stay siblings. To
place the bundle elsewhere, pass `--out` to the bundler and load it with `AssetBundle::load_dir`:

```sh
topcoat asset bundle --release --out dist/assets
```

See [Assets](asset.md) for the bundle format and the full set of bundler options.

## A bundle only matches a binary built beside it

Asset IDs are hashed from the declaration's crate name, source file, and path. For generated assets
the path contains `OUT_DIR`, so the same source yields a different ID under a different target
directory. A bundle built in one directory does not serve a binary built in another, which rules out
building the bundle in CI and shipping it separately.

Build the binary and the bundle in the same job, from the same checkout, and archive them together:

```sh
cargo build --release
topcoat asset bundle --release
tar -czf app.tar.gz target/release/my-app target/release/assets
```

Rendering an `Asset` that is missing from the loaded bundle panics during render instead of
returning a 500, so a mismatch shows up as an empty reply. Loading the bundle with `.unwrap()` at
startup turns that into a boot failure you notice immediately:

```rust,ignore
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    router::{Router, RouterBuilderDiscoverExt, module_router},
};

pub fn router() -> Router {
    module_router!
        .discover()
        .assets(AssetBundle::load().unwrap())
        .build()
}
```

## Configure the bind address with the environment

`HOST` and `PORT` decide where the server listens. Unset, it binds `127.0.0.1:3000`.

```sh
HOST=127.0.0.1 PORT=8080 ./my-app
```

`Topcoat.toml` is a marker file for editor tooling such as
[`topcoat fmt`](cli/fmt.md). It configures nothing at runtime, so the bind address does not belong
in it.

## Terminate TLS in a reverse proxy

The router speaks plain HTTP and terminates no TLS, so a public deployment needs a reverse proxy in
front of it. Bind Topcoat to loopback and let the proxy own the certificate:

```sh
HOST=127.0.0.1 PORT=8080 ./my-app
```

`serve` accepts any listener, including a Unix socket, which keeps the app off the network entirely.
Socket permissions then become the access control for the socket:

```rust,ignore
use tokio::net::UnixListener;

let path = "/run/my-app.sock";
let _ = std::fs::remove_file(path);
let listener = UnixListener::bind(path)?;
topcoat::serve(listener, router).await
```

Topcoat sends no `Strict-Transport-Security` header, so add one at the proxy. It also does not trust
forwarding headers by default: `client_ip` returns the peer's address unless you configure
`trusted_proxies`. See [Tower](router/tower.md) for both.

## Persist the cookie key across restarts

A `Key` generated per boot invalidates every signed and private cookie, so one restart logs out
every session. Generate the key once, store it with the rest of your secrets, and read it at startup
so restarts and replicas share it. `Key::from` takes the raw master bytes and requires at least 64
of them; use `Key::try_from` to get an error instead of a panic when the stored value is short:

```rust,ignore
use topcoat::cookie::Key;

fn cookie_key() -> Key {
    // Read the master key from the environment, where your secret manager
    // places it, so every restart and every replica uses the same one. Any
    // random value of at least 64 bytes works.
    let master = std::env::var("TOPCOAT_COOKIE_KEY").expect("TOPCOAT_COOKIE_KEY must be set");
    Key::try_from(master.as_bytes()).expect("TOPCOAT_COOKIE_KEY must be at least 64 bytes")
}
```

Register the result once so signed and private cookies share it:

```rust,ignore
module_router!()
    .discover()
    .cookies()
    .app_context(cookie_key())
    .build()
```

Whatever backs your [sessions](session.md) has to outlive the process for the same reason.

## Give shards a stable path

A `#[shard]` without a path gets a random one generated during compilation, so it changes on every
build. An open tab polling a shard gets a 404 after a restart and the widget goes silent. Pass an
absolute path to pin it:

```rust,ignore
#[shard("/search/results")]
async fn search_results(query: String) -> Result<impl View> { /* ... */ }
```

The browser re-renders that shard with a `POST` to `/search/results`, which the proxy forwards like
any other request. See [Shards](runtime/shard.md) for the path syntax.

## Shut down gracefully

`serve` handles `SIGINT`, and `SIGTERM` on Unix, by closing the accept loop and letting in-flight
requests finish until `RouterService::shutdown_timeout` expires. Point your supervisor at `SIGTERM`
so a deploy drains instead of cutting connections.
