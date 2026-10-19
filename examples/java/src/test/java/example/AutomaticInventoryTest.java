package example;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import static org.junit.jupiter.api.Assertions.*;

@ExtendWith(InventoryExtension.class)
class AutomaticInventoryTest {
    private Inventory inventory;

    @BeforeEach void stockProducts(Inventory.Events events) {
        inventory = new Inventory(events);
        inventory.addProduct("book", 6);
        inventory.reserve("book", "fixture", 1);
        inventory.addProduct("pen", 2);
    }

    @AfterEach void releaseFixtureReservation() {
        inventory.release("book", "fixture");
    }

    @Test void productKeysKeepOrdersIndependent() {
        inventory.reserve("book", "alice", 4);
        inventory.reserve("pen", "alice", 2);
        inventory.release("book", "alice");
        assertEquals(5, inventory.available("book"));
        assertEquals(0, inventory.available("pen"));
    }

    @Test void eachTestGetsFreshStock() {
        inventory.reserve("book", "alice", 5);
        assertEquals(0, inventory.available("book"));
        assertThrows(IllegalArgumentException.class, () -> inventory.reserve("book", "bob", 1));
    }
}
