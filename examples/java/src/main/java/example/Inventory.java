package example;

import io.github.crowecawcaw.prs.emit.PrsEmit;
import java.util.HashMap;
import java.util.Map;

/** Emits at the successful commit boundary; rejected requests emit nothing. */
public final class Inventory {
    private record Reservation(String sku, String order) {}
    private final Map<String, Integer> stock = new HashMap<>();
    private final Map<Reservation, Integer> reservations = new HashMap<>();

    public void addProduct(String sku, int units) {
        if (stock.containsKey(sku) || units < 0) throw new IllegalArgumentException("invalid product");
        stock.put(sku, units);
        PrsEmit.emit("eStocked", sku, Map.of("units", units));
    }

    public int available(String sku) {
        if (!stock.containsKey(sku)) throw new IllegalArgumentException("unknown product");
        return stock.get(sku);
    }

    public void reserve(String sku, String order, int units) {
        var key = new Reservation(sku, order);
        if (units <= 0 || reservations.containsKey(key) || units > available(sku))
            throw new IllegalArgumentException("invalid reservation");
        stock.put(sku, available(sku) - units);
        reservations.put(key, units);
        PrsEmit.emit("eReserved", sku, Map.of("order", order, "units", units));
    }

    public void release(String sku, String order) {
        var key = new Reservation(sku, order);
        var units = reservations.remove(key);
        if (units == null) throw new IllegalArgumentException("unknown reservation");
        stock.put(sku, available(sku) + units);
        PrsEmit.emit("eReleased", sku, Map.of("order", order, "units", units));
    }
}
