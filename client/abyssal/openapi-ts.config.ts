import { defineConfig } from "@hey-api/openapi-ts";

export default defineConfig({
    input: "./openapi.json",
    output: "src/client/generated",
    parser: {
        transforms: {
            enums: "root",
        },
    },
    plugins: ["@hey-api/client-axios"],
});
