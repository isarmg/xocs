import { xcssFontLicenses, xssiRuntimeInventory } from "./font-licenses.mjs";
import {defineConfig} from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig(async () => ({
  plugins: [react(), xcssFontLicenses(), ...await xssiRuntimeInventory()],
  server: {port: 5173, proxy: {'/api': 'http://127.0.0.1:8081'}},
}));
