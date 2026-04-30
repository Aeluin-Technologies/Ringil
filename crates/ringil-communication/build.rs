fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    prost_build::Config::new()
        .out_dir(out_dir)
        .compile_protos(&["./proto/zenoh.proto"], &["./proto"])
        .unwrap();
    println!("cargo:rerun-if-changed=./proto/zenoh.proto");
}
