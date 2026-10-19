package example;

import io.github.crowecawcaw.prs.PrsExtension;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import static org.junit.jupiter.api.Assertions.*;

// Settings come from prs.properties. Each test is recorded from before @BeforeEach until
// after @AfterEach, then checked.
@ExtendWith(PrsExtension.class)
class AutomaticInventoryTest {
    private Inventory inventory;

    @BeforeEach void stockProducts() {
        inventory = new Inventory();
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
