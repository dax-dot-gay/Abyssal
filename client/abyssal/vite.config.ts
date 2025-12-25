import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import { readFileSync } from "node:fs";
import { join } from "node:path";

// https://vite.dev/config/
export default defineConfig({
    plugins: [react()],
    server: {
        https: {
            key: readFileSync(join("certs", "key.pem")),
            cert: readFileSync(join("certs", "cert.pem")),
        },
        port: 5173,
        strictPort: true,
        proxy: {
            "/api": {
                target: "https://localhost:5174",
                changeOrigin: true,
                rewrite: (path) => path.replace(/^\/api/, ""),
                secure: false,
            },
        },
    },
});
