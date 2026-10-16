import {readFileSync} from "node:fs";
import {fileURLToPath, pathToFileURL} from "node:url";
import {join} from "node:path";
export async function xssiRuntimeInventory() {
  if (!process.env.XSSI_BUILD_TOOLS_DIR) return [];
  if (!process.env.XSSI_FRONTEND_METADATA_FILE) throw new Error("Explicit runtime inventory output is required");
  const {runtimeInventory} = await import(pathToFileURL(join(process.env.XSSI_BUILD_TOOLS_DIR,"scripts/frontend_build.mjs")));
  return [runtimeInventory({root:fileURLToPath(new URL("./",import.meta.url)),output:process.env.XSSI_FRONTEND_METADATA_FILE})];
}
export function xcssFontLicenses(prefix = "assets/") {
  const directory = new URL("./", import.meta.resolve("@xcss/web/web-fonts/OFL.txt"));
  return {
    name: "xcss-font-licenses",
    generateBundle() {
      for (const name of ["OFL.txt", "CJK-LICENSE.txt", "NORMAL-LICENSE.txt"]) {
        this.emitFile({type: "asset", fileName: prefix + name, source: readFileSync(new URL(name, directory))});
      }
    },
  };
}

export function leptosBuildMetadata() {
  return {
    name: "leptos-build-metadata",
    generateBundle(_options, bundle) {
      for (const chunk of Object.values(bundle)) {
        if (chunk.type === "chunk" && Object.keys(chunk.modules).some(id => /node_modules\/(?:react|react-dom|@vitejs\/plugin-react)(?:\/|$)/.test(id))) {
          this.error("React runtime must not enter the Leptos production bundle");
        }
      }
      this.emitFile({type: "asset", fileName: "assets/leptos-build.json", source: readFileSync(new URL("./pkg/leptos-build.json", import.meta.url))});
    },
  };
}
