mod a_sync;
mod exception;
mod sync;
mod utils;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/toy_jni.rs"));
}
