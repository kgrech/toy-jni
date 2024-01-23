package com.github.kgrech.toyjni.async;

import com.github.kgrech.toyjni.JNILoader;
import com.github.kgrech.toyjni.ToyJNIException;
import com.github.kgrech.toyjni.proto.Request;
import com.github.kgrech.toyjni.proto.Response;

import java.util.concurrent.CompletableFuture;

public class JNIBridge extends JNILoader implements AutoCloseable {

    private final long runtime;

    public JNIBridge() {
        this.runtime = init();
    }

    public Response.Success nativeCallBlocking(Request request) throws ToyJNIException {
        byte[] serializedRequest = request.toByteArray();
        byte[] response = nativeCallBlocking(serializedRequest);
        Response deserializedResponse;
        try {
            deserializedResponse = Response.parseFrom(response);
        } catch (Exception e) {
            throw new ToyJNIException("Protobuf schema mismatch", e);
        }
        if (deserializedResponse.hasError()) {
            throw new ToyJNIException(deserializedResponse.getError().getErrorMessage());
        }
        return deserializedResponse.getSuccess();
    }

    public CompletableFuture<Response.Success> nativeCall(Request request) throws ToyJNIException {
        byte[] serializedRequest = request.toByteArray();
        CompletableFuture<byte[]> future = nativeCall(serializedRequest);
        return future.thenApply(response -> {
            Response.Success deserializedResponse;
            try {
                deserializedResponse = Response.Success.parseFrom(response);
            } catch (Exception e) {
                throw new RuntimeException("Protobuf schema mismatch", e);
            }
            return deserializedResponse;
        });
    }

    @Override
    public native void close();

    private native long init();

    private native byte[] nativeCallBlocking(byte[] request);

    private native CompletableFuture<byte[]> nativeCall(byte[] request) throws ToyJNIException;

}
