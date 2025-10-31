# Pull Request

## Description

<!-- Provide a clear and concise description of your changes -->

### Type of Change

<!-- Mark the relevant option with an 'x' -->

- [ ] `feat`: New feature (non-breaking change which adds functionality)
- [ ] `fix`: Bug fix (non-breaking change which fixes an issue)
- [ ] `docs`: Documentation only changes
- [ ] `style`: Code style changes (formatting, missing semicolons, etc.)
- [ ] `refactor`: Code refactoring (neither fixes a bug nor adds a feature)
- [ ] `perf`: Performance improvements
- [ ] `test`: Adding or updating tests
- [ ] `build`: Changes to build system or dependencies
- [ ] `ci`: Changes to CI configuration files and scripts
- [ ] `chore`: Other changes that don't modify src or test files

### Breaking Changes

<!-- Mark with 'x' if this PR introduces breaking changes -->

- [ ] This PR includes breaking changes (requires major version bump)

<!-- If breaking changes, describe what breaks and how users should migrate -->

**Breaking Change Details:**
<!-- Leave empty if no breaking changes -->

## Motivation and Context

<!-- Why is this change required? What problem does it solve? -->
<!-- If it fixes an open issue, please link to the issue here -->

Closes #<!-- issue number -->

## How Has This Been Tested?

<!-- Describe the tests you ran to verify your changes -->

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Benchmarks added/updated (if performance-related)
- [ ] Manual testing performed

**Test Configuration:**
- OS: <!-- e.g., Ubuntu 22.04, macOS 13, Windows 11 -->
- Rust version: <!-- e.g., 1.70.0 -->

## Performance Impact

<!-- For performance-related changes, include benchmark results -->
<!-- Before/after comparisons are especially helpful -->

**Benchmarks:**
<!-- Leave empty if not applicable -->

## Checklist

<!-- Mark completed items with an 'x' -->

### Code Quality

- [ ] My code follows the style guidelines of this project
- [ ] I have run `cargo fmt` and code is properly formatted
- [ ] I have run `cargo clippy` and fixed all warnings
- [ ] I have run `make lint` and all checks pass
- [ ] My changes generate no new warnings

### Testing

- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] All new and existing tests pass locally (`cargo test`)
- [ ] I have run integration tests if applicable

### Documentation

- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have added/updated doc comments for public APIs
- [ ] I have updated the README.md if needed
- [ ] I have updated CONTRIBUTING.md if development workflow changed

### Commits

- [ ] My commit messages follow the [Conventional Commits](https://www.conventionalcommits.org/) specification
- [ ] Each commit is atomic and represents a single logical change
- [ ] Commit messages are clear and descriptive

**Example commit messages:**
```
feat: add support for .syncignore files
fix: handle symlinks correctly during sync
docs: update README with benchmark results
```

### Dependencies

- [ ] I have not added new dependencies without justification
- [ ] If new dependencies added, I have documented why they are needed
- [ ] Dependencies are minimal and well-maintained

### Performance

- [ ] I have considered performance implications of my changes
- [ ] No unnecessary allocations or copies introduced
- [ ] Streaming I/O is used where appropriate
- [ ] Benchmarks run successfully (if applicable)

## Additional Notes

<!-- Any additional information that reviewers should know -->

## Screenshots (if applicable)

<!-- Add screenshots for UI changes or progress output examples -->

---

**Reviewer Checklist:**

- [ ] Code changes reviewed
- [ ] Tests are adequate
- [ ] Documentation is clear
- [ ] Commit messages follow conventions
- [ ] CI passes
- [ ] No merge conflicts