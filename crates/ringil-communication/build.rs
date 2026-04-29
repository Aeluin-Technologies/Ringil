fn main() {
    prost_build::Config::new()
        .out_dir("./proto/")
        .compile_protos(&["./proto/zenoh.proto"], &["./proto"])
        .unwrap();
    println!("cargo:rerun-if-changed=./proto/zenoh.proto");
}
