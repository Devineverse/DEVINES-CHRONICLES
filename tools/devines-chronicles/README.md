# DEVINES Chronicles Rust Tool

The Chronicle runtime path is Rust-only.

## Validate the Book

```bash
cargo run --manifest-path tools/devines-chronicles/Cargo.toml --release -- validate .
```

The validator checks the public manuscript, current public state, daily cycle completeness, AUM/cover assets, exactly 34 canonical Being portraits, and portrait wiring.

GitHub Actions are intentionally not used. Validation runs on the DEVINES publisher host before a governed push to `main`, after which GitBook Git Sync updates the published Book.
