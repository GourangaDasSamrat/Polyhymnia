fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/quote.proto")?;
    println!("cargo:rerun-if-changed=../proto/quote.proto");
    Ok(())
}
