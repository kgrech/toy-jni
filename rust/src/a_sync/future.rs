use anyhow::{Context, Error};
use jni::objects::{GlobalRef, JObject, JThrowable, JValue};
use jni::JNIEnv;

const FUTURE_CLASS: &str = "java/util/concurrent/CompletableFuture";
const FUTURE_CONSTRUCTOR: &str = "()V";
const COMPLETE_METHOD_NAME: &str = "complete";
const COMPLETE_METHOD_SIGNATURE: &str = "(Ljava/lang/Object;)Z";
const COMPLETE_EXCEPTIONALLY_METHOD_NAME: &str = "completeExceptionally";
const COMPLETE_EXCEPTIONALLY_METHOD_SIGNATURE: &str = "(Ljava/lang/Throwable;)Z";

#[derive(Clone)]
pub struct ToyJniFuture {
    /// Pointer to java future object. GlobalRef garbage collection
    java_future: GlobalRef,
}

impl ToyJniFuture {
    pub fn new(env: &mut JNIEnv) -> anyhow::Result<Self> {
        let java_future_obj = env.new_object(FUTURE_CLASS, FUTURE_CONSTRUCTOR, &[])?;
        let java_future = env.new_global_ref(java_future_obj)?;
        Ok(Self { java_future })
    }

    pub fn complete(&self, env: &mut JNIEnv, object: &JObject) -> anyhow::Result<bool> {
        let response = env
            .call_method(
                self.java_future.as_obj(),
                COMPLETE_METHOD_NAME,
                COMPLETE_METHOD_SIGNATURE,
                &[JValue::from(object)],
            )
            .with_context(|| {
                format!("Error calling {COMPLETE_METHOD_NAME}({COMPLETE_METHOD_SIGNATURE}) method")
            })?;
        response
            .z()
            .with_context(|| format!("{COMPLETE_METHOD_NAME} method returned non-bool value"))
    }

    pub fn complete_exceptionally(
        &self,
        env: &mut JNIEnv,
        object: &JThrowable,
    ) -> anyhow::Result<bool> {
        let response = env
            .call_method(
                self.java_future.as_obj(),
                COMPLETE_EXCEPTIONALLY_METHOD_NAME,
                COMPLETE_EXCEPTIONALLY_METHOD_SIGNATURE,
                &[JValue::from(object)],
            )
            .with_context(|| {
                format!(
                    "Error calling {COMPLETE_EXCEPTIONALLY_METHOD_NAME}({COMPLETE_EXCEPTIONALLY_METHOD_SIGNATURE}) method"
                )
            })?;
        response.z().with_context(|| {
            format!("{COMPLETE_EXCEPTIONALLY_METHOD_NAME} method returned non-bool value")
        })
    }

    pub fn local_reference<'a>(&self, env: &JNIEnv<'a>) -> Result<JObject<'a>, Error> {
        env.new_local_ref(&self.java_future)
            .with_context(|| "Unable to conver to java future global reference to local reference")
    }
}
