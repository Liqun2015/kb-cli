# Contributing Guide

Thank you for your interest in contributing to kb-cli!

## How to Contribute

### Reporting Issues

Please report bugs or request features using GitHub Issues.

### Submitting Code

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Commit your changes: `git commit -m 'Add some feature'`
4. Push to the branch: `git push origin feature/your-feature`
5. Open a Pull Request

## Code Style

- Use `cargo fmt` to format code
- Use `cargo clippy` to check code
- Add necessary tests

## Development

```bash
cargo build
cargo test
cargo run -- --help
```

## Testing

Test your changes before submitting:
- Ensure `cargo build` succeeds
- Run `cargo test` if applicable
- Test the specific functionality you changed

## Questions?

Feel free to open a discussion in GitHub Issues if you have questions.
