# Contributing to Google Cast & Nest Display Caster

Thank you for your interest in contributing to the Google Cast & Nest Display Caster project! Contributions from the community help make this utility more robust, feature-rich, and efficient.

## Code of Conduct

Please be respectful and constructive in all communication, including issues, pull requests, and discussions.

## How to Contribute

### 1. Reporting Bugs
- Search existing issues to see if the bug has already been reported.
- If not, create a new issue with a clear title and description, including:
  - Your environment (OS version, Rust version, `yt-dlp` version).
  - The model of your Google Cast device (e.g. Nest Hub, Nest Display, Chromecast Ultra).
  - Clear steps to reproduce the issue.
  - Actual vs. expected behavior.

### 2. Suggesting Features
- Open a new issue describing the feature you would like to see, why it is useful, and how it might be implemented.

### 3. Pull Requests
- Fork the repository and create a new branch for your change (e.g. `feature/my-new-feature` or `fix/some-bug`).
- Keep changes concise, focused, and well-documented.
- Ensure that the project compiles cleanly and matches standard formatting rules:
  ```bash
  cargo fmt --check
  cargo clippy --all-targets
  cargo check
  ```
- Write clear, descriptive commit messages.
- Open a Pull Request pointing to our `main` branch.

## Development Guidelines

- **Rust Standards**: Ensure you are using the latest stable Rust compiler.
- **Error Handling**: Use `anyhow` for main application errors, giving descriptive context to failures.
- **Code Styling**: Follow the standard styling verified by `cargo fmt`.

## License

By contributing to this project, you agree that your contributions will be licensed under the project's MIT License.
