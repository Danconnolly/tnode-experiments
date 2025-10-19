# Claude Code Instructions for Teranode Experiments

## Project Overview

This is a Rust workspace for experimenting with BSV Blockchain Association's Teranode. It contains:
- `teranode-client`: Core gRPC client library
- `tnode-lab`: Experimental command-line tool (`tnode`) for interacting with Teranode

## Development Workflow

### Before Making Changes

1. Always run `cargo check` before starting work to ensure the project compiles
2. When adding new proto files, rebuild with `cargo build` to regenerate gRPC code
3. Check existing code patterns in the crate before adding new functionality

### Code Standards

- **Async**: All I/O operations must use Tokio async/await
- **Error Handling**:
  - Use `anyhow::Result` for application-level errors
  - Use `thiserror` for library error types in `teranode-client`
  - Never use `.unwrap()` or `.expect()` in library code
  - Use `?` operator for error propagation
- **Logging**: Use `tracing` macros (`info!`, `debug!`, `warn!`, `error!`) instead of `println!`
- **Naming Conventions**:
  - Snake_case for functions and variables
  - PascalCase for types and traits
  - SCREAMING_SNAKE_CASE for constants

### Proto Files

- Proto files are stored in `crates/teranode-client/proto/`
- Generated code goes to `crates/teranode-client/src/proto/` (gitignored)
- Never manually edit generated proto code
- After adding/modifying proto files, always rebuild: `cargo build`
- Include proto modules in `lib.rs` under the `proto` module

### Testing

- Write unit tests in the same file as the code using `#[cfg(test)]` modules
- Write integration tests in `tests/` directory
- Mock external gRPC calls in tests
- Run tests with: `cargo test`
- Run tests with logs: `RUST_LOG=debug cargo test -- --nocapture`

### Building and Running

```bash
# Check code without building
cargo check

# Build all crates
cargo build

# Build optimized release
cargo build --release

# Run the CLI
cargo run --bin tnode -- --help

# Run specific binary with args
cargo run --bin tnode -- --endpoint http://localhost:50051 ping
```

### Documentation

- Add doc comments (`///`) for all public APIs
- Include examples in doc comments when helpful
- Document error conditions and panics
- Run `cargo doc --open` to view documentation

### Code Quality

- Run `cargo fmt` before committing (already handled by git hooks per global CLAUDE.md)
- Run `cargo clippy` before committing (already handled by git hooks per global CLAUDE.md)
- Address all clippy warnings - don't suppress without good reason
- Keep functions focused and small (prefer < 50 lines)
- Prefer explicit types over type inference in public APIs

### Adding New Features

When adding new functionality:

1. Start with the public API design in the library
2. Implement the core logic in `teranode-client`
3. Add CLI commands in `tnode-lab` that use the library
4. Write tests for both library and CLI
5. Update README.md with new functionality
6. Add doc comments

### gRPC Client Patterns

- Create client connections in async context
- Use `tonic::transport::Channel` for connection pooling
- Handle `tonic::Status` errors appropriately
- Implement retry logic for transient failures
- Set reasonable timeouts on requests
- Use streaming RPCs where appropriate

### Error Handling Patterns

```rust
// In teranode-client library
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TeranodeError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),
}

// In application code
use anyhow::{Context, Result};

fn do_something() -> Result<()> {
    let result = operation()
        .context("failed to perform operation")?;
    Ok(result)
}
```

### Workspace Dependencies

- All common dependencies are defined in workspace `Cargo.toml`
- Use `dependency.workspace = true` in crate `Cargo.toml` files
- Add new dependencies to workspace first, then reference them

### CLI Development

- Use `clap` derive macros for command-line parsing
- Provide helpful error messages and usage examples
- Support `--verbose` flag for debug logging
- Default to sensible values (e.g., `localhost:50051` for endpoint)
- Print progress for long-running operations

### Performance Considerations

- Use `cargo build --release` for performance testing
- Profile with `cargo flamegraph` if needed
- Consider connection pooling for gRPC clients
- Reuse `Channel` instances rather than creating new connections
- Use streaming where appropriate for large data transfers

### Common Tasks

**Adding a new gRPC service client:**
1. Add proto file to `crates/teranode-client/proto/`
2. Rebuild: `cargo build`
3. Import generated code in `lib.rs`: `pub mod proto { include!("proto/service.rs"); }`
4. Create wrapper client in `client.rs`
5. Add CLI command in `tnode-lab/src/main.rs`

**Adding a new CLI command:**
1. Add variant to `Commands` enum in `main.rs`
2. Implement handler in match statement
3. Use library functions from `teranode-client`
4. Update README.md usage section

**Adding a new library feature:**
1. Design public API
2. Implement in appropriate module
3. Export from `lib.rs`
4. Add tests
5. Document with `///` comments

## Project-Specific Notes

- This is experimental code - prioritize clarity over optimization initially
- Focus on correct gRPC communication patterns
- Keep CLI simple and user-friendly
- Library should be reusable in other projects
- Expect proto definitions to change - keep client code flexible

## When in Doubt

- Check existing code patterns in the workspace
- Refer to Tonic documentation for gRPC patterns: https://github.com/hyperium/tonic
- Follow Rust API guidelines: https://rust-lang.github.io/api-guidelines/
- Ask for clarification rather than making assumptions about Teranode behavior
