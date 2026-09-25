# Contributing to ClipFlow

Thank you for your interest in contributing!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/clipflow.git`
3. Install dependencies: `pnpm install`
4. Start development: `pnpm tauri dev`

## Development Workflow

### Branching

- `main` - Stable releases
- `develop` - Integration branch
- Feature branches: `feat/short-description`
- Fix branches: `fix/short-description`
- Docs branches: `docs/short-description`

### Commits

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add color picker to settings
fix: prevent duplicate clipboard items
docs: update architecture diagram
refactor: extract color parser module
test: add pipeline integration tests
chore: update dependencies
```

### Code Style

**Frontend (TypeScript/Svelte)**

- Run `pnpm lint` and `pnpm format` before committing
- Use `svelte-check` for type checking
- Follow existing patterns in `src/lib/features/`

**Backend (Rust)**

- Run `cargo fmt` and `cargo clippy` before committing
- No `unwrap()`/`expect()` on external input
- Use `anyhow::Result` for errors
- Document public APIs with `///`

### Testing

**Frontend**

```bash
pnpm test              # Unit tests
pnpm test:ui           # With UI
```

**Backend**

```bash
cd src-tauri
cargo test             # All tests
cargo test --test integration  # Integration tests only
```

**Full Test Suite**

```bash
# In CI: runs on every PR
pnpm lint && pnpm format --check && pnpm svelte-check
cd src-tauri && cargo check && cargo clippy && cargo test
```

## Pull Request Process

1. Ensure all checks pass
2. Update documentation if needed
3. Add tests for new functionality
4. Request review from maintainers
5. Address feedback
6. Squash and merge (maintainers)

## Reporting Issues

Use GitHub Issues with:

- Clear title and description
- Steps to reproduce
- Expected vs actual behavior
- Platform (OS, version)
- ClipFlow version
- Logs/screenshots if relevant

## Security Issues

Report via [GitHub Security Advisories](https://github.com/clipflow/clipflow/security/advisories) - NOT public issues.

## Code of Conduct

Be respectful, inclusive, and constructive. See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Questions?

Open a Discussion or ask in the issue tracker.
