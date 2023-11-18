package com.github.kgrech.toyjni;

public class ToyJNIException extends Exception {

    public ToyJNIException() {
    }

    public ToyJNIException(String message) {
        super(message);
    }

    public ToyJNIException(String message, Throwable cause) {
        super(message, cause);
    }

    public ToyJNIException(Throwable cause) {
        super(cause);
    }
}
