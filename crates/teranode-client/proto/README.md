# Protobuf Definitions

Place Teranode `.proto` files in this directory.

## How to add proto files

1. Copy `.proto` files from the Teranode repository to this directory
2. The build script (`build.rs`) will automatically compile them during build
3. Generated Rust code will be available in the `teranode_client::proto` module

## Expected workflow

```bash
# Example: copying proto files from Teranode repo
cp /path/to/teranode/proto/*.proto crates/teranode-client/proto/

# Build the project (this will compile the proto files)
cargo build
```

The generated code will be included in `src/proto/` and can be used via:

```rust
use teranode_client::proto;
```
