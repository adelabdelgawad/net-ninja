# Contributing to NetNinja

Thanks for your interest in contributing! This guide covers how to get set up and submit changes.

## Development Setup

1. Install prerequisites: Rust 1.77.2+, Node.js 18+, and [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
2. Clone the repo and install frontend dependencies:
   ```bash
   git clone https://github.com/AdelDima/netninja.git
   cd netninja/src/frontend
   npm install
   ```
3. Start the dev environment:
   ```bash
   npm run tauri:dev
   ```

## Branching

- Create feature branches from `main`: `git checkout -b feat/my-feature`
- Use prefixes: `feat/`, `fix/`, `docs/`, `refactor/`, `chore/`

## Pull Request Process

1. Ensure `cargo check` passes in `src/backend`
2. Ensure `npx tsc --noEmit` passes in `src/frontend`
3. Write a clear PR description explaining **what** and **why**
4. Keep PRs focused -- one logical change per PR
5. Link related issues if applicable

## Code Style

- **Rust**: Follow standard `rustfmt` conventions. Run `cargo fmt` before committing.
- **TypeScript**: Follow the existing project style. Use TypeScript strict mode.
- **CSS**: Tailwind utility classes. Avoid custom CSS where possible.

## Adding a Tauri Command

When adding a new IPC command, update **three** files:

1. `src/backend/src/adapters/tauri/*.rs` -- define the `#[tauri::command]` function
2. `src/backend/src/adapters/tauri/mod.rs` -- add to `generate_handler![]`
3. `src/frontend/src-tauri/src/lib.rs` -- add to `generate_handler![]`

## Reporting Issues

Use the [GitHub issue templates](.github/ISSUE_TEMPLATE/) for bug reports and feature requests.

## Code of Conduct

This project follows the [Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
