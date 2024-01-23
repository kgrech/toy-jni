use crate::proto::{Request, Response};
use crate::utils::{handle_error, MessageExt};
use anyhow::Context;
use jni::objects::{GlobalRef, JByteArray, JObject, JValue};
use jni::{JNIEnv, JavaVM};
use std::time::Duration;

fn handle_request(
    j_object: &JObject,
    env: &mut JNIEnv,
    request: Request,
) -> anyhow::Result<Response> {
    let response = if request.message == "Hello, Rust!" {
        if request.response_delay == 0 {
            Response::success("Hello, Java!".into())
        } else {
            let context = CallbackContext::new(env, j_object)?;
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(request.response_delay));
                let response = context
                    .callback(Request::message("Hello, Java!".into()))
                    .expect("Callback error!");
                println!("Response from callback: {response:?}")
            });
            Response::success("Will reply later!".into())
        }
    } else {
        Response::error(format!("Unable to respond to '{}'", request.message))
    };
    Ok(response)
}

#[no_mangle]
pub extern "system" fn Java_com_github_kgrech_toyjni_sync_JNIBridge_nativeCall<'a>(
    env: JNIEnv<'a>,
    j_object: JObject<'a>,
    request: JByteArray<'a>,
) -> JByteArray<'a> {
    handle_error(env, |env| {
        let request = Request::decode_from_java(env, &request)?;
        let response = handle_request(&j_object, env, request)?;
        response.encode_to_java(env)
    })
}

#[derive(Debug)]
pub struct CallbackContext {
    jvm: JavaVM,
    global_ref: GlobalRef,
}

impl CallbackContext {
    pub fn new(env: &JNIEnv, j_object: &JObject) -> anyhow::Result<Self> {
        let global_ref = env
            .new_global_ref(j_object)
            .with_context(|| "Unable to create global reference for java object repo")?;
        let jvm = env
            .get_java_vm()
            .with_context(|| "Unable to JVM reference")?;
        Ok(Self { jvm, global_ref })
    }

    pub fn callback(&self, response: Request) -> anyhow::Result<Response> {
        // Attach current thread to the JVM
        let mut guard = self
            .jvm
            .attach_current_thread()
            .with_context(|| "Unable to attach current thread to the JVM")?;

        let data = response.encode_to_java(&guard)?;

        // Call java back
        let response = guard
            .call_method(
                self.global_ref.as_obj(),
                "onJniCallback",
                "([B)[B",
                &[JValue::from(&data)],
            )
            .with_context(|| "Error calling onJniCallback method")?;

        // Convert response to JObject
        let j_object = response
            .l()
            .with_context(|| "onJniCallback returned non-object")?;
        let j_byte_array: JByteArray = j_object.into();
        Response::decode_from_java(&mut guard, &j_byte_array)
    }
}
