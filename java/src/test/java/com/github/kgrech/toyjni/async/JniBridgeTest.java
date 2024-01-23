package com.github.kgrech.toyjni.async;

import com.github.kgrech.toyjni.ToyJNIException;
import com.github.kgrech.toyjni.proto.Request;
import com.github.kgrech.toyjni.proto.Response;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

public class JniBridgeTest {

    @Test
    public void testNativeCallBlockingSuccess() throws ToyJNIException {
        try (JNIBridge bridge = new JNIBridge()) {
            Response.Success response = bridge.nativeCallBlocking(Request.newBuilder().setMessage("Hello, Rust!").build());
            Assertions.assertEquals(response.getMessage(), "Hello, Java!");
        }
    }

    @Test
    public void testNativeCallBlockingError() {
        try (JNIBridge bridge = new JNIBridge()) {
            ToyJNIException exception = Assertions.assertThrows(
                    ToyJNIException.class,
                    () -> bridge.nativeCallBlocking(Request.newBuilder().setMessage("Bad greeting").build()));
            Assertions.assertEquals("Unable to respond to 'Bad greeting'", exception.getMessage());
        }

    }

    @Test
    @Timeout(value = 10)
    public void testNativeCallSuccess() throws ExecutionException, InterruptedException, ToyJNIException {
        try (JNIBridge bridge = new JNIBridge()) {
            Request request = Request.newBuilder()
                    .setMessage("Hello, Rust!")
                    .setResponseDelay(1000)
                    .build();
            CompletableFuture<Response.Success> future = bridge.nativeCall(request);
            Response.Success response = future.get();
            Assertions.assertEquals(response.getMessage(), "Hello, Java!");
        }
    }

    @Test
    @Timeout(value = 10)
    public void testNativeCallError() throws ToyJNIException {
        try (JNIBridge bridge = new JNIBridge()) {
            Request request = Request.newBuilder()
                    .setMessage("Bad greeting!")
                    .setResponseDelay(1000)
                    .build();
            CompletableFuture<Response.Success> future = bridge.nativeCall(request);
            ExecutionException exception = Assertions.assertThrows(
                    ExecutionException.class,
                    future::get);
            ToyJNIException toyJNIException = (ToyJNIException) exception.getCause();
            Assertions.assertEquals("Unable to respond to 'Bad greeting!'", toyJNIException.getMessage());
        }
    }

}
