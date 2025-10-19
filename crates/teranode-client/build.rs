//! Build script for compiling protobuf definitions

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from("proto");

    // Check if proto directory exists and contains .proto files
    if !proto_dir.exists() {
        println!("cargo:warning=Proto directory not found. Create 'proto' directory and add .proto files.");
        return Ok(());
    }

    // Collect all .proto files in the proto directory
    let proto_files: Vec<PathBuf> = std::fs::read_dir(&proto_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "proto" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    if proto_files.is_empty() {
        println!("cargo:warning=No .proto files found in proto directory.");
        println!("cargo:warning=Add protobuf files from Teranode to crates/teranode-client/proto/");
        return Ok(());
    }

    println!("cargo:rerun-if-changed=proto");

    // Compile proto files with tonic
    tonic_build::configure()
        .build_server(false) // We only need the client
        .build_client(true)
        .out_dir("src/proto") // Output generated code to src/proto
        .compile_protos(&proto_files, &[proto_dir])?;

    Ok(())
}
