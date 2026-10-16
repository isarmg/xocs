import {leptosStaticAssets} from "./leptos-assets.mjs";
import { leptosBuildMetadata, xcssFontLicenses, xssiRuntimeInventory } from "./font-licenses.mjs";
import {defineConfig} from 'vite';

export default defineConfig(async () => ({
  plugins: [leptosStaticAssets(), leptosBuildMetadata(), xcssFontLicenses(), ...await xssiRuntimeInventory()],
  build: { target: "esnext", cssTarget: "esnext", modulePreload: { polyfill: false }, sourcemap: false },
  server: {port: 5173, proxy: {'/api': 'http://127.0.0.1:8081'}},
}));
