package example;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.RegisterExtension;
import java.nio.file.Path;
import io.github.crowecawcaw.prs.PrsExtension;
import io.github.crowecawcaw.prs.Recording;
import static org.junit.jupiter.api.Assertions.*;

class AutomaticInventoryTest {
    @RegisterExtension static final PrsExtension PRS = PrsExtension.builder()
            .specFiles(Path.of("specs/inventory.p")).specs("InventoryCorrect")
            .coverage(Path.of("target/coverage/monitor.json")).build();
    private Inventory inventory;

    // The injected Recording is open before @BeforeEach and checked after @AfterEach.
    @BeforeEach void stockProducts(Recording events) {
        inventory = new Inventory((sku, event, payload) -> events.emit(event, sku, payload));
        inventory.addProduct("coffee", 3);
        inventory.addProduct("tea", 2);
        inventory.reserve("coffee", "fixture", 1);
    }

    @AfterEach void releaseFixtureReservation() {
        inventory.release("coffee", "fixture");
    }

    @Test void productKeysKeepOrdersIndependent() {
        inventory.reserve("coffee", "order-1", 2);
        inventory.reserve("tea", "order-1", 2);
        inventory.release("coffee", "order-1");
        assertEquals(2, inventory.available("coffee"));
        assertEquals(0, inventory.available("tea"));
    }

    @Test void eachTestGetsFreshStock() {
        inventory.reserve("coffee", "order-1", 2);
        assertEquals(0, inventory.available("coffee"));
        assertThrows(IllegalArgumentException.class, () -> inventory.reserve("coffee", "order-2", 1));
    }
}
