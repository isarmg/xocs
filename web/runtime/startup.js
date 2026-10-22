// Keep startup failures visible even when the Rust/WASM module cannot load.
// Successful application rendering and font readiness remain owned by Rust.
export function startApplication(initialize) {
  const html = document.documentElement;
  if (html.dataset.blogStartup) throw new Error("Application startup already began");
  html.dataset.blogStartup = "pending";
  let finish;
  const completed = new Promise(resolve => { finish = resolve; });
  let settled = false;
  const timeout = setTimeout(() => fail("timeout"), 30_000);
  const ready = () => {
    if (html.dataset.blogRuntime !== "blog-interface" || html.dataset.blogFonts !== "ready") {
      fail("runtime");
      return;
    }
    if (!settle()) return;
    html.dataset.blogStartup = "ready";
    finish(true);
  };
  const failed = () => fail("resources");
  const scriptError = event => fail("runtime", event.error ?? event.reason);
  function settle() {
    if (settled) return false;
    settled = true;
    clearTimeout(timeout);
    document.removeEventListener("blog-startup-ready", ready);
    document.removeEventListener("blog-startup-failed", failed);
    window.removeEventListener("error", scriptError);
    window.removeEventListener("unhandledrejection", scriptError);
    return true;
  }
  function fail(stage, error) {
    if (!settle()) return;
    html.dataset.blogStartup = "failed";
    html.dataset.blogFonts = "failed";
    delete html.dataset.blogRuntime;
    const language = new URL(window.location.href).searchParams.get("lang") ?? html.lang;
    const chinese = language.toLowerCase().startsWith("zh");
    const panel = document.createElement("section");
    panel.id = "blog-startup-error";
    panel.setAttribute("role", "alert");
    panel.setAttribute("aria-labelledby", "blog-startup-error-title");
    panel.dataset.stage = stage;
    panel.tabIndex = -1;
    const title = document.createElement("h1");
    title.id = "blog-startup-error-title";
    title.textContent = chinese ? "页面启动失败" : "Page could not start";
    const message = document.createElement("p");
    message.textContent = chinese
      ? "页面所需资源加载失败或超时。请检查网络连接，然后重新加载页面。"
      : "Required page resources failed to load or timed out. Check your connection, then reload the page.";
    const retry = document.createElement("button");
    retry.type = "button";
    retry.textContent = chinese ? "重新加载页面" : "Reload page";
    retry.addEventListener("click", () => {
      retry.disabled = true;
      window.location.reload();
    }, { once: true });
    panel.append(title, message, retry);
    document.body.append(panel);
    panel.focus({ preventScroll: true });
    console.error("Application startup failed", stage, error ?? "");
    finish(false);
  }
  document.addEventListener("blog-startup-ready", ready);
  document.addEventListener("blog-startup-failed", failed);
  window.addEventListener("error", scriptError);
  window.addEventListener("unhandledrejection", scriptError);
  Promise.resolve().then(initialize).catch(error => fail("runtime", error));
  return completed;
}
