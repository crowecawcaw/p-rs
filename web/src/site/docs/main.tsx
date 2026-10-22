import { hydrateIslands } from "../islands";
import "../../styles.css";
import "../site.css";
import "./docs.css";

hydrateIslands({
  loops: () => import("./diagrams/Loops").then((m) => m.Loops),
  schedules: () => import("./diagrams/Schedules").then((m) => m.Schedules),
  trace: () => import("./diagrams/Trace").then((m) => m.Trace),
  workflow: () => import("./diagrams/Workflow").then((m) => m.Workflow),
});
