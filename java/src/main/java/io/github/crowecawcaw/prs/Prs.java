package io.github.crowecawcaw.prs;

import java.lang.annotation.ElementType;
import java.lang.annotation.Inherited;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

/**
 * Per-test options for {@link PrsExtension}, on a test method or class (the method wins).
 * In {@code marked} mode only annotated tests are recorded.
 */
@Target({ElementType.METHOD, ElementType.TYPE})
@Retention(RetentionPolicy.RUNTIME)
@Inherited
public @interface Prs {
    /** In {@link #skip()}: skip every spec, so the test is not checked. */
    String ALL = "*";

    /** Specs this test does not check, or {@link #ALL}. */
    String[] skip() default {};

    /** Also fail when a key ends in a hot state. */
    boolean checkHot() default false;
}
