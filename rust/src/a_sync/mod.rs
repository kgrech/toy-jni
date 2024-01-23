mod callback;
mod future;

use crate::a_sync::callback::CallbackContext;
use crate::a_sync::future::ToyJniFuture;
use crate::proto::{Request, Response};
use crate::utils::{borrow_field, get_field, handle_error, handle_error_exceptionally, MessageExt};
use jni::objects::{JByteArray, JObject};
use jni::sys::jlong;
use jni::JNIEnv;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

const RUNTIME_FIELD: &str = "runtime";

async fn async_handle_request(request: Request) -> Response {
    if request.response_delay > 0 {
        tokio::time::sleep(Duration::from_millis(request.response_delay)).await;
    }
    let response = if request.message == "Hello, Rust!" {
        Response::success("Hello, Java!".into())
    } else {
        Response::error(format!("Unable to respond to '{}'", request.message))
    };
    response
}

#[no_mangle]
pub extern "system" fn Java_com_github_kgrech_toyjni_async_JNIBridge_init<'a>(
    _env: JNIEnv<'a>,
    _j_object: JObject<'a>,
) -> jlong {
    let runtime = Arc::new(Runtime::new().expect("Unable to init runtime"));
    Arc::into_raw(runtime) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_github_kgrech_toyjni_async_JNIBridge_nativeCallBlocking<'a>(
    env: JNIEnv<'a>,
    j_object: JObject<'a>,
    request: JByteArray<'a>,
) -> JByteArray<'a> {
    handle_error(env, |env| unsafe {
        let request = Request::decode_from_java(env, &request)?;
        let runtime: Arc<Runtime> = borrow_field(env, &j_object, RUNTIME_FIELD)?;
        let response = runtime.block_on(async_handle_request(request));
        response.encode_to_java(env)
    })
}

#[no_mangle]
pub extern "system" fn Java_com_github_kgrech_toyjni_async_JNIBridge_nativeCall<'a>(
    env: JNIEnv<'a>,
    j_object: JObject<'a>,
    request: JByteArray<'a>,
) -> JObject<'a> {
    handle_error_exceptionally(env, |env| unsafe {
        let request = Request::decode_from_java(env, &request)?;
        let runtime: Arc<Runtime> = borrow_field(env, &j_object, RUNTIME_FIELD)?;

        let future = ToyJniFuture::new(env)?;
        let local_ref = future.local_reference(env)?;
        let callback_context = CallbackContext::new(env, future)?;
        runtime.spawn(async move {
            let response = async_handle_request(request).await;
            callback_context
                .callback(response)
                .expect("Unrecoverable callback error");
        });
        Ok(local_ref)
    })
    .unwrap_or_default()
}

#[no_mangle]
pub extern "system" fn Java_com_github_kgrech_toyjni_async_JNIBridge_close<'a>(
    mut env: JNIEnv<'a>,
    j_object: JObject<'a>,
) {
    unsafe {
        let runtime = get_field::<Runtime>(&mut env, &j_object, RUNTIME_FIELD);
        drop(runtime);
    }
}
