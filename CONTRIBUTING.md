# Contributing

Keep it focused, clean, and useful.

## Ground Rules

- Ship small, reviewable pull requests.
- Prefer clarity over cleverness.
- Keep the UI responsive; never block the main thread with Docker calls.
- Preserve current product direction: simple desktop control panel, no bloat.

## Local Setup

```bash
cargo check
cargo test
```

If you touch behavior, add or update tests when practical.

## Code Style

- Rust 2021 idioms, straightforward naming.
- Keep modules cohesive (`ui`, `docker`, `model`).
- Return actionable errors; avoid panics in user paths.
- Keep README in sync with shipped behavior.

## Pull Request Checklist

- [ ] The app compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Changes are scoped and intentional
- [ ] README/docs updated if needed
- [ ] No unrelated refactors

## Commit Style

Use clear, imperative commit messages:

- `Add container restart action`
- `Improve error handling for docker start`
- `Refine README quick start section`
