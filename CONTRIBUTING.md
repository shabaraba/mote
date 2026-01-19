# Contributing to mote

Thank you for considering contributing to mote! 🎉

We welcome contributions of all kinds: bug reports, feature requests, documentation improvements, and code contributions.

## Code of Conduct

This project is committed to providing a welcoming and inclusive environment for all contributors.

### Expected Behavior

- Be respectful and considerate in your communication
- Welcome diverse perspectives and experiences
- Accept constructive criticism gracefully
- Focus on what is best for the community and project
- Show empathy towards other community members

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Personal attacks or trolling
- Public or private harassment
- Publishing others' private information without permission
- Other conduct which could reasonably be considered inappropriate

### Reporting

If you experience or witness unacceptable behavior, please report it by opening an issue or contacting the project maintainers directly. All reports will be handled with discretion and confidentiality.

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

- A clear and descriptive title
- Detailed steps to reproduce the issue
- Expected behavior vs actual behavior
- Your environment (OS, Rust version, mote version)
- Any relevant logs or error messages

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:

- A clear and descriptive title
- A detailed description of the proposed functionality
- Any possible implementation approaches
- Why this enhancement would be useful

### Pull Requests

We actively welcome your pull requests!

1. **Fork the repository** and create your branch from `main`
2. **Name your branch** descriptively (e.g., `feat/add-export-command`, `fix/restore-permission-error`)
3. **Make your changes** following the coding standards below
4. **Add tests** for your changes if applicable
5. **Ensure all tests pass**: `cargo test`
6. **Run formatting and linting**:
   ```bash
   cargo fmt
   cargo clippy -- -D warnings
   ```
7. **Update documentation** if you changed functionality
8. **Write a clear commit message** following the commit message guidelines
9. **Submit your PR** with a clear description of the changes

#### Commit Message Guidelines

We use [Semantic Commit Messages](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code formatting (no functional changes)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Build process or tooling changes

Examples:
```
feat: add --quiet flag to snapshot command
fix: handle empty directories in restore
docs: update README with new examples
```

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Git

### Building from Source

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/mote.git
cd mote

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

## Coding Standards

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for code formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Write idiomatic Rust code
- Add documentation comments for public APIs

### Code Organization

- Keep functions focused and small
- Use meaningful variable and function names
- Add comments for complex logic
- Organize code into logical modules

### Error Handling

- Use `anyhow::Result` for functions that can fail
- Provide clear error messages
- Use `thiserror` for custom error types

### Testing

- Write unit tests for new functionality
- Add integration tests for CLI commands
- Test edge cases and error conditions
- Ensure tests are deterministic

## Project Structure

```
mote/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── cli.rs            # CLI argument parsing
│   ├── config.rs         # Configuration handling
│   ├── error.rs          # Error types
│   ├── ignore.rs         # File ignore logic
│   └── storage/          # Storage backend
│       ├── mod.rs
│       ├── object.rs     # Content-addressable object storage
│       ├── snapshot.rs   # Snapshot metadata
│       └── ...
├── tests/                # Integration tests
├── docs/                 # Documentation
├── scripts/              # Build and release scripts
└── examples/             # Usage examples
```

## Documentation

### Code Documentation

- Add doc comments (`///`) for public APIs
- Include examples in doc comments when helpful
- Document function parameters and return values

### User Documentation

- Update README.md for user-facing changes
- Add examples for new features
- Keep documentation clear and concise

## Release Process

Releases are handled by maintainers. See [docs/development/RELEASE.md](docs/development/RELEASE.md) for details.

## Getting Help

Need help? We're here for you:

- 💬 **GitHub Discussions**: Ask questions and share ideas
- 🐛 **Issues**: Report bugs or request features
- 📖 **Documentation**: Check the [docs/](docs/) directory

## Recognition

Contributors will be recognized in:
- The project's README
- Release notes
- Git history

## License

By contributing to mote, you agree that your contributions will be licensed under the MIT License.
