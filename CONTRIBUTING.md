# Contributing to Spiris Bokföring och Fakturering API Client

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Code of Conduct

Be respectful and constructive in all interactions. We're here to build great software together.

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Git
- A Spiris Bokföring och Fakturering account (for testing)

### Development Setup

1. Fork the repository on GitHub

2. Clone your fork:
```bash
git clone https://github.com/YOUR_USERNAME/claude_jungle_bamboo
cd claude_jungle_bamboo
```

3. Create a new branch:
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

4. Install dependencies:
```bash
cargo build
```

5. Run tests to ensure everything works:
```bash
cargo test
```

## Development Workflow

### Making Changes

1. **Write Tests First**: If you're adding new functionality, write tests first
2. **Follow Rust Conventions**: Use `cargo fmt` and `cargo clippy`
3. **Update Documentation**: Add or update documentation for any changes
4. **Keep Commits Focused**: Each commit should represent a single logical change

### Code Style

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check with clippy
cargo clippy -- -D warnings

# Ensure all tests pass
cargo test
```

### Commit Messages

Follow conventional commits format:

```
type(scope): brief description

Longer description if needed

- Detail 1
- Detail 2
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Examples:
```
feat(endpoints): add support for suppliers endpoint

Add complete CRUD operations for suppliers:
- List suppliers with pagination
- Get supplier by ID
- Create new supplier
- Update existing supplier
- Delete supplier
```

```
fix(retry): correct exponential backoff calculation

The backoff multiplier was not being applied correctly,
resulting in linear backoff instead of exponential.
```

## Testing

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Documentation tests
cargo test --doc
```

### Writing Tests

Add tests for all new functionality:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        let input = "test";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

## Documentation

### Code Documentation

Use Rust doc comments:

```rust
/// Brief description of the function.
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `param1` - Description of parameter
/// * `param2` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// This function will return an error if...
///
/// # Example
///
/// ```
/// use spiris::Client;
///
/// let result = function(param1, param2);
/// ```
pub fn function(param1: Type1, param2: Type2) -> Result<Type3> {
    // Implementation
}
```

### README Updates

If you add new features, update:
- README.md with usage examples
- CHANGELOG.md with the changes
- Examples directory if appropriate

## Pull Request Process

### Before Submitting

1. **Ensure all tests pass**: `cargo test`
2. **Format your code**: `cargo fmt`
3. **Run clippy**: `cargo clippy -- -D warnings`
4. **Update documentation**: Document new features
5. **Update CHANGELOG.md**: Add your changes under "Unreleased"
6. **Rebase on main**: Keep your branch up to date

### Submitting

1. Push your branch to your fork:
```bash
git push origin feature/your-feature-name
```

2. Create a Pull Request on GitHub

3. Fill out the PR template with:
   - Description of changes
   - Related issues (if any)
   - Testing performed
   - Breaking changes (if any)

### PR Requirements

- All tests must pass
- Code must be formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
- Documentation updated
- CHANGELOG.md updated

### Review Process

1. A maintainer will review your PR
2. Address any requested changes
3. Once approved, your PR will be merged

## Adding New Endpoints

When adding support for new API endpoints:

1. **Add types** in `src/types.rs`:
```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NewResource {
    pub id: Option<String>,
    pub name: Option<String>,
    // ... other fields
}
```

2. **Create endpoint module** in `src/endpoints/new_resource.rs`:
```rust
use crate::client::Client;
use crate::error::Result;
use crate::types::{NewResource, PaginatedResponse, PaginationParams};

pub struct NewResourceEndpoint<'a> {
    client: &'a Client,
}

impl<'a> NewResourceEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self, params: Option<PaginationParams>) -> Result<PaginatedResponse<NewResource>> {
        // Implementation
    }

    // ... other methods
}
```

3. **Add to endpoints module** in `src/endpoints/mod.rs`:
```rust
pub mod new_resource;
pub use new_resource::NewResourceEndpoint;
```

4. **Add accessor to Client** in `src/lib.rs`:
```rust
impl Client {
    pub fn new_resources(&self) -> endpoints::NewResourceEndpoint<'_> {
        endpoints::NewResourceEndpoint::new(self)
    }
}
```

5. **Add tests** in your endpoint module
6. **Add example** in `examples/` directory
7. **Update README.md** with usage examples

## Release Process

(For maintainers)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` - move "Unreleased" to version number
3. Commit: `git commit -m "chore: release v0.2.0"`
4. Tag: `git tag -a v0.2.0 -m "Release v0.2.0"`
5. Push: `git push && git push --tags`
6. Publish: `cargo publish`

## Questions?

If you have questions or need help:

1. Check existing issues and discussions
2. Open a new issue with your question
3. Tag it with the "question" label

## Thank You!

Your contributions make this project better for everyone. Thank you for taking the time to contribute!
