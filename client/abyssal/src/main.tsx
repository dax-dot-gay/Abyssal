import "@mantine/core/styles.css";
import "@mantine/dates/styles.css";
import "@mantine/dropzone/styles.css";
import "@mantine/code-highlight/styles.css";
import "@mantine/notifications/styles.css";
import "@mantine/spotlight/styles.css";

import "./styles/index.scss";
import "./util/theme/theme.css";

import { createRoot } from "react-dom/client";
import { AbyssalRoot } from "./root";

createRoot(document.getElementById("root")!).render(<AbyssalRoot />);
