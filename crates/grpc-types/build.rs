use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = "../../grpc";

    let proto_files = ["telemetry.proto"];

    let proto_paths: Vec<PathBuf> = proto_files
        .iter()
        .map(|file| PathBuf::from(proto_root).join(file))
        .collect();

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .compile_protos(&proto_paths, &[proto_root.into()])?;

    for file in proto_files {
        println!("cargo:rerun-if-changed={proto_root}/{file}");
    }

    Ok(())
}
