use std::env;

fn main() {
    let root_dir = env::current_dir().unwrap();
    let proto_dir = root_dir.parent().unwrap().join("protobuf");
    let proto_path = proto_dir.join("toy_jni.proto");
    println!("cargo:rerun-if-changed={}", proto_path.to_str().unwrap());

    prost_build::compile_protos(&[proto_path], &[proto_dir]).expect("protoc error");
}
