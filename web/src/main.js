import "../xcss-web/vendor/xcss-web/startup.css";
import { startApplication } from "../xcss-web/vendor/xcss-web/startup.js";
// Application state, requests, validation and views are implemented in Rust.
import "@xcss/web/design-tokens/tokens.css";
import "@xcss/web/design-tokens/tokens.dark.css";
import "@xcss/web/admin-ui/styles.css";
import "@xcss/web/design-tokens/reset.css";
import "@xcss/web/design-tokens/accessibility.css";
import "@xcss/web/web-fonts/fonts.css";
import { startServerCorners } from "@xcss/web/admin-ui/server-corners";
import "./style.css";
import "./public-original.css";
import "./public-layout.css";
import "./admin.css";
import wasmUrl from "../pkg/xocs_web_bg.wasm?url";
await startApplication(async () => {
  startServerCorners(document);
  const { default: init } = await import("../pkg/xocs_web.js");
  await init({ module_or_path: wasmUrl });
});
