# Contributing to Beefcake

Thank you for your interest in contributing to Beefcake!

## How to Contribute

1. Fork the repository.
2. Create a new branch for your feature or bug fix.
3. Make your changes and ensure tests pass.
4. Submit a pull request.

## Development Setup

- Install Rust and Cargo.
- Install Node.js and npm.
- Run `npm install` in the project root.
- Run `cargo build` to build the backend.
- Run `npm run dev` to start the frontend development server.

## Code Style

### Rust Code
- Follow the existing code style in the project.
- Use `cargo fmt` to format Rust code.
- Run `cargo clippy` to catch common mistakes.

### TypeScript Code
- **ESLint**: Catches bugs and enforces code quality rules.
- **Prettier**: Ensures consistent formatting across all files.
- **Pre-commit hooks**: Automatically run linting and formatting before commits.

### Code Quality Commands

```bash
# Format TypeScript code
npm run format

# Check formatting without modifying files
npm run format:check

# Lint TypeScript code
npm run lint

# Fix linting issues automatically
npm run lint:fix

# Type check without building
npm run type-check

# Run all quality checks at once
npm run quality
```

## Pre-commit Hooks

This project uses [Husky](https://typicode.github.io/husky/) to run automated checks before each commit:

1. **ESLint**: Fixes linting issues automatically
2. **Prettier**: Formats code automatically
3. **lint-staged**: Only checks files you've modified (fast)

### First-time Setup

After cloning the repository and running `npm install`, Husky will be automatically installed.

### Bypassing Hooks (Use Sparingly)

If you need to commit without running pre-commit hooks (not recommended):

```bash
git commit --no-verify -m "Your message"
```

**Note**: CI will still run all checks, so bypassing hooks locally doesn't skip validation.

## VS Code Setup (Recommended)

For the best development experience, install these VS Code extensions:

- **ESLint** (`dbaeumer.vscode-eslint`)
- **Prettier** (`esbenp.prettier-vscode`)
- **Vitest** (`ZixuanChen.vitest-explorer`)

The repository includes workspace settings (`.vscode/settings.json`) that will:
- Auto-format files on save
- Auto-fix ESLint errors on save
- Organize imports automatically

## Running Tests

See [TESTING.md](TESTING.md) for comprehensive testing documentation.

Quick reference:

```bash
# Frontend tests
npm test                # Run unit tests
npm run test:coverage   # With coverage report
npm run test:e2e        # End-to-end tests

# Rust tests
cargo test              # Run all Rust tests
npm run test:rust       # Alternative via npm

# All tests
npm run test:all        # Runs everything
```

## Code Review Checklist

Before submitting a PR, ensure:

- ✅ All tests pass (`npm run test:all`)
- ✅ No linting errors (`npm run lint`)
- ✅ Code is formatted (`npm run format:check`)
- ✅ Type checking passes (`npm run type-check`)
- ✅ Documentation is updated (if adding features)
- ✅ Test coverage is maintained or improved
