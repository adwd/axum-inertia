axum-inertia
============

Implementation of the [inertia.js] protocol for axum.

This repository is a fork of [mjhoy/axum-inertia].

Provides an `Inertia` axum extractor to render responses like so:

```rust
async fn get_posts(i: Inertia) -> impl IntoResponse {
    i.render("Posts/Index", json!({ "posts": vec!["post one", "post two"] }))
}
```

See [crate documentation] for more information.

## Example

The [`todo` example](examples/todo) is a complete Axum, Inertia, React, and
Vite application. Its README includes instructions for development and
production builds.

[inertia.js]: https://inertiajs.com
[mjhoy/axum-inertia]: https://github.com/mjhoy/axum-inertia
[crate documentation]: https://docs.rs/axum-inertia/latest/axum_inertia/

## Making a new release

1. Spin off a `bump-vX.X.X` branch
2. Update the `CHANGELOG`; start a new `[Unreleased]` section
3. Bump the version number in `Cargo.toml`
4. Run `cargo build` (this updates `Cargo.lock`
5. Run `cargo release` (this will run a dry-run, requires [cargo-release][cargo-release])
6. Merge PR
7. Update `main` branch locally and run `cargo release --execute`

[cargo-release]: https://github.com/crate-ci/cargo-release
