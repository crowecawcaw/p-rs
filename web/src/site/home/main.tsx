import { hydrateIslands } from "../islands";
import "../../styles.css";
import "../site.css";
import "./home.css";

hydrateIslands({ speed: () => import("./SpeedBars").then((m) => m.SpeedBars) });
