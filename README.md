# toy-jni
Rust JNI bindings to play with


## Setup
```
brew install protobuf
protoc protobuf/toy_jni.proto --java_out=./java/src/main/java
```

## Run
```
cd rust
cargo build
cd ../java
./gradlew test
```