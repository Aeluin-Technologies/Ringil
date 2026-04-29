fn main() {
    prost_build::Config::new()
        .out_dir("./proto/")
        .compile_protos(&["./proto/zenoh.proto"], &["./proto"])
        .unwrap();
}
