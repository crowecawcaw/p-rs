package io.github.crowecawcaw.snappea;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/** Qualifies Recording injection when a test registers more than one SnappeaExtension. */
@Target(ElementType.PARAMETER)
@Retention(RetentionPolicy.RUNTIME)
public @interface SnappeaRecording {
    String value();
}
