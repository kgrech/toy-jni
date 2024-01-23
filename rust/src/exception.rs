use anyhow::Context;
use jni::objects::{JThrowable, JValue};
use jni::JNIEnv;
use std::fmt::Display;

const TOY_JNI_EXCEPTION: &str = "com/github/kgrech/toyjni/ToyJNIException";
const TOY_JNI_EXCEPTION_CONSTRUCTOR: &str = "(Ljava/lang/String;)V";

pub fn throw_exception(env: &mut JNIEnv, error: impl Display) {
    // Check if there is an existing Java exception
    match env.exception_check() {
        Ok(true) => {
            // If there is an existing exception, return here to re-throw it
        }
        Ok(false) => {
            let result = env.throw_new(TOY_JNI_EXCEPTION, error.to_string());
            if let Err(err) = result {
                env.fatal_error(format!("Error throwing {TOY_JNI_EXCEPTION}: {err}"))
            }
        }
        Err(err) => env.fatal_error(format!("Exception check failed: {err}")),
    }
}

pub fn create_exception<'a>(
    env: &mut JNIEnv<'a>,
    error: impl Display,
) -> anyhow::Result<JThrowable<'a>> {
    let java_sting = env
        .new_string(error.to_string())
        .with_context(|| "Error converting error string to Java string")?;
    let j_object = env
        .new_object(
            TOY_JNI_EXCEPTION,
            TOY_JNI_EXCEPTION_CONSTRUCTOR,
            &[JValue::from(&java_sting)],
        )
        .with_context(|| format!("Error creating {TOY_JNI_EXCEPTION}"))?;
    Ok(j_object.into())
}
