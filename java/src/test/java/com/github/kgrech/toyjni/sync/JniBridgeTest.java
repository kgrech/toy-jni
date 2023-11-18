package com.github.kgrech.toyjni.sync;

import com.github.kgrech.toyjni.ToyJNIException;
import com.github.kgrech.toyjni.proto.Request;
import com.github.kgrech.toyjni.proto.Response;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;

public class JniBridgeTest {

    @Test
    public void testNativeCallSuccess() throws ToyJNIException {
        JNIBridge bridge = new JNIBridge();
        Response.Success response = bridge.nativeCall(Request.newBuilder().setMessage("Hello, Rust!").build());
        Assertions.assertEquals(response.getMessage(), "Hello, Java!");
    }

    @Test
    public void testNativeCallError() {
        JNIBridge bridge = new JNIBridge();
        ToyJNIException exception = Assertions.assertThrows(
                ToyJNIException.class,
                () -> bridge.nativeCall(Request.newBuilder().setMessage("Bad greeting").build()));
        Assertions.assertEquals("Unable to respond to 'Bad greeting'", exception.getMessage());
    }

    @Test
    @Timeout(value = 10)
    public void testCallback() throws ToyJNIException, InterruptedException {
        TestJNIBridge bridge = new TestJNIBridge();
        Request request = Request.newBuilder()
                .setMessage("Hello, Rust!")
                .setResponseDelay(1000)
                .build();
        Response.Success response = bridge.nativeCall(request);
        Assertions.assertEquals(response.getMessage(), "Will reply later!");
        bridge.latch.await();
        Assertions.assertEquals(bridge.request.getMessage(), "Hello, Java!");
    }


    private static class TestJNIBridge extends JNIBridge {
        public final CountDownLatch latch = new CountDownLatch(1);
        public Request request = null;

        @Override
        public Response onJniCallback(Request request) {
            this.request = request;
            latch.countDown();
            return super.onJniCallback(request);
        }
    }
}
