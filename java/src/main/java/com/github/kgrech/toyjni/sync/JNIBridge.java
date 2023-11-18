package com.github.kgrech.toyjni.sync;

import com.github.kgrech.toyjni.ToyJNIException;
import com.github.kgrech.toyjni.proto.Request;
import com.github.kgrech.toyjni.proto.Response;

import java.util.concurrent.atomic.AtomicInteger;

public class JNIBridge {

    static {
        System.loadLibrary("toy_jni");
    }

    public Response.Success nativeCall(Request request) throws ToyJNIException {
        byte[] serializedRequest = request.toByteArray();
        byte[] response = nativeCall(serializedRequest);
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

    public Response onJniCallback(Request request) {
        System.out.println("Native callback: " + request.getMessage());
        Response.Success success = Response.Success.newBuilder().setMessage("Hello from the callback").build();
        return Response.newBuilder().setSuccess(success).build();
    }


    private native byte[] nativeCall(byte[] request);

    private byte[] onJniCallback(byte [] request) {
        try {
            Request deserializedRequest = Request.parseFrom(request);
            Response response = onJniCallback(deserializedRequest);
            return response.toByteArray();
        } catch (Exception e) {
            return Response
                    .newBuilder()
                    .setError(Response.Error.newBuilder().setErrorMessage(e.getMessage()))
                    .build()
                    .toByteArray();
        }
    }
}
