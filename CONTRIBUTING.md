# Contributing to AfricasTalking-rs

First off, thank you for your interest in improving AfricasTalking-rs! We welcome contributions of all kinds: bug reports, feature requests, documentation improvements, code refactors, tests, examples, and more.

---

## Table of Contents

- [Contributing to AfricasTalking-rs](#contributing-to-africastalking-rs)
  - [Table of Contents](#table-of-contents)
  - [How to Contribute](#how-to-contribute)
  - [Coding Guidelines](#coding-guidelines)
  - [Development Workflow](#development-workflow)
  - [Commit Message Format](#commit-message-format)
  - [Testing](#testing)
  - [Pull Request Checklist](#pull-request-checklist)
  - [Code of Conduct](#code-of-conduct)

---

## How to Contribute

1. **Fork** the repository.
2. **Clone** your fork locally:
   ```sh
   git clone https://github.com/MikeTeddyOmondi/africastalking-rs.git
   cd africastalking-rs
   ```

3. **Create a branch**:

   ```sh
   git checkout -b feat/my-feature
   ```
4. Make your changes in your branch.
5. **Run tests** and ensure all pass:

   ```sh
   cargo test
   cargo fmt -- --check
   cargo clippy
   ```
6. **Commit** your changes following the [Commit Message Format](#commit-message-format).
7. **Push** your branch and open a **Pull Request** against `main`.

---

## Coding Guidelines

* Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
* Use the builder pattern for request structs.
* Document all public items with `///` comments.
* Keep modules focused: one API domain per file (`sms.rs`, `airtime.rs`, etc.).
* Write clear, concise, and idiomatic Rust.
* Run `cargo fmt` to automatically format code.
* Run `cargo clippy` to catch common mistakes and style issues.

---

## Development Workflow

* Branch off `main` for every feature or fix: `feat/...`, `fix/...`, `docs/...`.
* Rebase regularly to stay up to date with `main`.
* Squash or clean up commits before merging.
* Include unit tests for new functionality and bug fixes.
* Update `CHANGELOG.md` as needed (see [Keep a Changelog](https://keepachangelog.com/)).

---

## Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/) style:

```
<type>(<scope>): <short summary>

<body (optional)>

<footer (optional, e.g., “Closes #123” or “BREAKING CHANGE: ...”)>
```

**Types**:

* `feat`: a new feature
* `fix`: a bug fix
* `docs`: documentation only changes
* `refactor`: code change that neither fixes a bug nor adds a feature
* `test`: adding missing tests or correcting existing ones
* `chore`: maintenance tasks (CI, build scripts, etc.)

---

## Testing

* Write unit tests in each module under `src/modules/*.rs`.
* Use `#[tokio::test]` for async tests.
* For HTTP interactions, use [`wiremock`](https://github.com/LukeMathWalker/wiremock-rs) or [mockito](https://github.com/lipanski/mockito).
* Ensure tests cover error scenarios and edge cases.

---

## Pull Request Checklist

* [ ] My code follows the project’s style guidelines
* [ ] I have added or updated tests and they all pass
* [ ] I have added or updated documentation where necessary
* [ ] I have updated `CHANGELOG.md` with relevant changes
* [ ] I have addressed any code review feedback

---

## Code of Conduct

This project follows the [Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you agree to abide by its terms.

---

Thank you for helping make AfricasTalking-rs better! 🚀

