use crate::exception::throw_exception;
use crate::proto::response::{Error, Success};
use crate::proto::{Request, Response};
use anyhow::Context;
use jni::objects::{JByteArray, JObject, JValueOwned, ReleaseMode};
use jni::signature::Primitive;
use jni::sys::jlong;
use jni::JNIEnv;
use prost::Message;
use std::ffi::c_void;
use std::sync::Arc;

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

pub fn handle_error_exceptionally<'a, T>(
    mut env: JNIEnv<'a>,
    f: impl FnOnce(&mut JNIEnv<'a>) -> anyhow::Result<T>,
) -> Option<T> {
    match f(&mut env) {
        Ok(obj) => Some(obj),
        Err(error) => {
            throw_exception(&mut env, error);
            None
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

/// Converts the value of the field into Arc without incrementing the Arc's ref counter.
pub unsafe fn get_field<T>(
    env: &mut JNIEnv,
    j_object: &JObject,
    field_name: &str,
) -> anyhow::Result<Arc<T>> {
    let field_value: JValueOwned = env
        .get_field(j_object, field_name, Primitive::Long.to_string())
        .with_context(|| format!("Can't get long value of the '{field_name}' field"))?;
    let j_long_value: jlong = field_value
        .j()
        .with_context(|| format!("Wrong type of the '{field_name}' field."))?;
    let ptr = j_long_value as *const c_void;
    Ok(Arc::from_raw(ptr.cast()))
}

/// Converts the value of the field into Arc and increments the Arc's ref counter
pub unsafe fn borrow_field<T>(
    env: &mut JNIEnv,
    j_object: &JObject,
    field_name: &str,
) -> anyhow::Result<Arc<T>> {
    let handle: Arc<T> = get_field(env, j_object, field_name)?;
    let clone = handle.clone();
    std::mem::forget(handle);
    Ok(clone)
}
