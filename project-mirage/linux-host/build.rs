use std::path::PathBuf;

fn main() {
    let proto_file = "../common/proto/mirage.proto";
    
    println!("cargo:rerun-if-changed={}", proto_file);

    prost_build::Config::new()
        .out_dir("src/proto")
        .compile_protos(&[proto_file], &["../common/proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos: {}", e));
}
