fn main() {
    tonic_build::configure()
        .compile_protos(&["proto/vfs.proto"], &["proto"])
        .unwrap();
}
