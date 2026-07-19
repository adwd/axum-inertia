# Todo example

A simple Todo application built with Rust, `axum`, `axum-inertia`, React, and
Vite. Todos are stored in server memory and reset whenever the server restarts.

## Development

Rust and Node.js are required. From the repository root, install the frontend
dependencies first:

```bash
npm --prefix examples/todo/client install
```

Then start Vite and Axum in separate terminals, also from the repository root:

```bash
npm --prefix examples/todo/client run dev
```

```bash
cargo run --manifest-path examples/todo/Cargo.toml
```

Open [http://localhost:3000](http://localhost:3000).

## Production

From the repository root:

```bash
npm --prefix examples/todo/client run build
APP_ENV=production cargo run --release --manifest-path examples/todo/Cargo.toml
```

In production mode, Axum serves the static assets from
`examples/todo/client/dist`.

## Tests

```bash
cargo test --manifest-path examples/todo/Cargo.toml
npm --prefix examples/todo/client run typecheck
```
