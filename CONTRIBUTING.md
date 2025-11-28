# Contributing to Arkitect

First off, thank you for considering contributing to Arkitect! It's people like you that make Arkitect such a great tool.

## Code of Conduct

This project and everyone participating in it is governed by our commitment to providing a welcoming and inspiring community for all.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues as you might find out that you don't need to create one. When you are creating a bug report, please include as many details as possible:

* **Use a clear and descriptive title**
* **Describe the exact steps which reproduce the problem**
* **Provide specific examples to demonstrate the steps**
* **Describe the behavior you observed after following the steps**
* **Explain which behavior you expected to see instead and why**
* **Include screenshots or animated GIFs** if applicable

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

* **Use a clear and descriptive title**
* **Provide a step-by-step description of the suggested enhancement**
* **Provide specific examples to demonstrate the steps**
* **Describe the current behavior** and **explain the expected behavior**
* **Explain why this enhancement would be useful**

### Pull Requests

* Fill in the required template
* Follow the Python and Rust style guides
* Include tests when adding new features
* Update documentation as needed
* End all files with a newline

## Development Setup

### Prerequisites

* Python 3.9+
* Rust 1.75+
* Git

### Setup Steps

```bash
# Clone the repository
git clone https://github.com/SH1W4/arkitect.git
cd arkitect

# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -e ".[dev]"

# Run tests
pytest

# Run linters
black arkitect/
isort arkitect/
flake8 arkitect/
mypy arkitect/
```

## Style Guides

### Python Style Guide

* Follow PEP 8
* Use Black for formatting (line length: 88)
* Use isort for import sorting
* Use type hints where possible
* Write docstrings for all public functions/classes

### Rust Style Guide

* Follow standard Rust conventions
* Use `cargo fmt` for formatting
* Use `cargo clippy` for linting
* Write documentation comments for public APIs

### Git Commit Messages

* Use the present tense ("Add feature" not "Added feature")
* Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
* Limit the first line to 72 characters or less
* Reference issues and pull requests liberally after the first line
* Use conventional commits format:
  * `feat:` - New feature
  * `fix:` - Bug fix
  * `docs:` - Documentation changes
  * `style:` - Code style changes (formatting, etc.)
  * `refactor:` - Code refactoring
  * `test:` - Adding or updating tests
  * `chore:` - Maintenance tasks

### Example Commit Messages

```
feat: add task scheduling optimization algorithm

Implements priority-based scheduling with deadline awareness.
Includes tests and documentation.

Closes #123
```

## Testing

* Write tests for all new features
* Ensure all tests pass before submitting PR
* Aim for high code coverage
* Include both unit and integration tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=arkitect --cov-report=html

# Run specific test file
pytest tests/test_scheduler.py
```

## Documentation

* Update README.md if adding new features
* Add/update docstrings for new functions and classes
* Update CHANGELOG.md following Keep a Changelog format
* Consider adding examples to `examples/` directory

## Review Process

1. Create a feature branch from `master`
2. Make your changes
3. Write/update tests
4. Update documentation
5. Submit a pull request
6. Address review feedback
7. Once approved, a maintainer will merge your PR

## Recognition

Contributors will be recognized in:
* CHANGELOG.md (for significant contributions)
* Project README (for major features)
* GitHub contributors page

## Questions?

Feel free to:
* Open an issue for questions
* Join discussions in GitHub Discussions
* Reach out to maintainers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Arkitect! 🚀
