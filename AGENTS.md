# Repository Guidelines

## Project Structure & Module Organization

This repository is a single Rust library crate implementing the Inertia.js protocol for Axum. The public entry point is `src/lib.rs`. Protocol concerns are split into focused modules: request parsing in `src/request.rs`, response construction in `src/response.rs`, page data in `src/page.rs`, partial reloads and props in `src/partial.rs` and `src/props.rs`, configuration in `src/config.rs`, and Vite integration in `src/vite.rs`. Tests are colocated with their modules in `#[cfg(test)]` blocks. GitHub Actions configuration lives in `.github/workflows/`; release notes belong in `CHANGELOG.md`.

## Build, Test, and Development Commands

- `cargo build --locked` compiles the crate using the committed lockfile.
- `cargo test --locked --all-features --all-targets` runs the same broad test suite used by CI.
- `cargo test --locked --all-features --doc` checks examples embedded in API documentation.
- `cargo fmt --check` verifies formatting; run `cargo fmt` to apply it.
- `cargo clippy --all-targets --all-features` runs the CI lints. Treat warnings as failures (`RUSTFLAGS="-Dwarnings"`).
- `cargo doc --open` builds and opens the crate documentation locally.

The crate targets Rust 1.75 or newer and edition 2021. Before submitting, run formatting, Clippy, and both test commands.

## Coding Style & Naming Conventions

Follow standard Rust conventions and the repository's `rustfmt.toml`. Use four-space indentation, `snake_case` for functions and modules, `CamelCase` for types and traits, and `SCREAMING_SNAKE_CASE` for constants. Keep modules focused and expose only intentional API through `pub` declarations. Public behavior should include concise `///` documentation and, where practical, a compiling example. Avoid unnecessary clones and unchecked failures in new request-handling paths.

## Testing Guidelines

Use built-in `#[test]` for synchronous logic and `#[tokio::test]` for async Axum behavior. Give tests behavior-focused names such as `it_responds_with_conflict_on_version_mismatch`. Place tests beside the code they cover; exercise headers, status codes, serialized page data, and relevant edge cases. There is no stated coverage threshold, but regressions and new public behavior require tests.

## Commit & Pull Request Guidelines

Write short, imperative commit subjects consistent with history, for example `add vite base option` or `fix query params in response url`. Keep each commit scoped to one concern. Pull requests should explain the behavioral change, identify compatibility or protocol implications, link related issues, and include tests. Update `CHANGELOG.md` under `[Unreleased]` for user-visible changes. Confirm stable/beta CI, formatting, Clippy, tests, and doctests pass; screenshots are only useful for documentation or rendered-output changes.
