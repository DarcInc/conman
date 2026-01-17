fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .out_dir("src/generated")
        .compile_protos(
            &[
                "contracts/hello_world.proto",
                "contracts/list_containers.proto",
            ],
            &["contracts"],
        )?;
    println!("cargo:rerun-if-changed=contracts/hello_world.proto");
    println!("cargo:rerun-if-changed=contracts/list_containers.proto");
    Ok(())
}
