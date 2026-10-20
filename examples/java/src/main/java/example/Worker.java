package example;

/**
 * A separate process: stocks a product and reserves it. Started by a test with
 * SNAPPEA_EVENTS_DIR set, its events go to a file of its own in that folder.
 * Usage: Worker <sku> <stock> <order> <units>
 */
public final class Worker {
    public static void main(String[] args) {
        var inventory = new Inventory();
        inventory.addProduct(args[0], Integer.parseInt(args[1]));
        inventory.reserve(args[0], args[2], Integer.parseInt(args[3]));
    }
}
