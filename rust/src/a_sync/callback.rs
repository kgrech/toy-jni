use crate::a_sync::future::ToyJniFuture;
use crate::exception::create_exception;
use crate::proto::Response;
use crate::utils::MessageExt;
use anyhow::Context;
use jni::{JNIEnv, JavaVM};

pub struct CallbackContext {
    jvm: JavaVM,
    future: ToyJniFuture,
}

impl CallbackContext {
    pub fn new(env: &JNIEnv, future: ToyJniFuture) -> anyhow::Result<Self> {
        let jvm = env
            .get_java_vm()
            .with_context(|| "Unable to JVM reference")?;
        Ok(Self { jvm, future })
    }

    pub fn callback(&self, response: Response) -> anyhow::Result<()> {
        // Attach current thread to the JVM
        let mut guard = self
            .jvm
            .attach_current_thread()
            .with_context(|| "Unable to attach current thread to the JVM")?;

        if let Some(success) = response.success {
            let data = success.encode_to_java(&guard)?;
            self.future.complete(&mut guard, &data)?;
        } else if let Some(error) = response.error {
            let exception = create_exception(&mut guard, error.error_message)?;
            self.future.complete_exceptionally(&mut guard, &exception)?;
        }
        Ok(())
    }
}
