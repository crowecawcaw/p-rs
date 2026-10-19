package example;

import org.junit.jupiter.api.extension.*;

/** Start before user @BeforeEach; verify after user @AfterEach, including fixture events. */
final class InventoryExtension implements BeforeEachCallback, AfterEachCallback, ParameterResolver {
    private static final ExtensionContext.Namespace NS = ExtensionContext.Namespace.create(InventoryExtension.class);
    @Override public void beforeEach(ExtensionContext context) throws Exception {
        var coverage = context.getRoot().getStore(NS).getOrComputeIfAbsent(
                "coverage", key -> new Coverage(), Coverage.class);
        context.getStore(NS).put("recording", new Recording(coverage));
    }
    @Override public void afterEach(ExtensionContext context) throws Exception {
        context.getStore(NS).remove("recording", Recording.class).close();
    }
    @Override public boolean supportsParameter(ParameterContext parameter, ExtensionContext context) {
        return parameter.getParameter().getType() == Inventory.Events.class;
    }
    @Override public Object resolveParameter(ParameterContext parameter, ExtensionContext context) {
        return context.getStore(NS).get("recording", Recording.class);
    }
}
