import "../runtime/startup.css";
import { startApplication } from "../runtime/startup.js";
// Application state, requests, validation and views are implemented in Rust.
import "@blog/styles/design-tokens/tokens.css";
import "@blog/styles/design-tokens/tokens.dark.css";
import "@blog/styles/admin-ui/styles.css";
import "@blog/styles/design-tokens/reset.css";
import "@blog/styles/design-tokens/accessibility.css";
import "@blog/styles/web-fonts/fonts.css";
import "./style.css";
import "./public-original.css";
import "./public-layout.css";
import "./admin.css";
import wasmUrl from "../pkg/blog_browser_bg.wasm?url";
await startApplication(async () => {
  const { default: init } = await import("../pkg/blog_browser.js");
  await init({ module_or_path: wasmUrl });
});
