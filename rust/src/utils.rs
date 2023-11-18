use crate::proto::response::{Error, Success};
use crate::proto::{Request, Response};
use anyhow::Context;
use jni::objects::{JByteArray, ReleaseMode};
use jni::JNIEnv;
use prost::Message;

pub trait MessageExt: Message + Default {
    fn encode_to_java<'a>(&self, env: &JNIEnv<'a>) -> anyhow::Result<JByteArray<'a>> {
        env.byte_array_from_slice(self.encode_to_vec().as_slice())
            .with_context(|| "Error converting request to java byte array")
    }
    fn decode_from_java(env: &mut JNIEnv, j_byte_array: &JByteArray) -> anyhow::Result<Self> {
        unsafe {
            let guard = env
                .get_array_elements_critical(j_byte_array, ReleaseMode::NoCopyBack)
                .with_context(|| "Failed to access byte array")?;
            let slice = &*guard;
            let u8_slice: &[u8] =
                std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len());
            Self::decode(u8_slice).with_context(|| "Unable to decode request")
        }
    }
}

impl<T: Message + Default> MessageExt for T {}

pub fn handle_error<'a>(
    mut env: JNIEnv<'a>,
    f: impl FnOnce(&mut JNIEnv<'a>) -> anyhow::Result<JByteArray<'a>>,
) -> JByteArray<'a> {
    match f(&mut env) {
        Ok(obj) => obj,
        Err(error) => {
            let response = Response::error(error.to_string());
            if let Ok(response_bytes) = response.encode_to_java(&env) {
                return response_bytes;
            }
            env.fatal_error(error.to_string());
        }
    }
}

impl Request {
    pub fn message(message: String) -> Self {
        #[allow(clippy::needless_update)]
        Self {
            message,
            ..Default::default()
        }
    }
}

impl Response {
    pub fn success(message: String) -> Self {
        #[allow(clippy::needless_update)]
        Self {
            success: Some(Success {
                message,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    pub fn error(error_message: String) -> Self {
        #[allow(clippy::needless_update)]
        Self {
            error: Some(Error {
                error_message,
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
