# Local HTTP console over the native replica

The console is a Vue SPA talking to a local Axum process that already owns the Euler replica. WASM was the alternative (one binary in the browser) but would re-host the same CPU-bound 1 s/step integration and split the library into a second target; a local backend keeps one native code path and matches the request for a Rust server. The SPA is static files, so the whole console still runs offline once built.
