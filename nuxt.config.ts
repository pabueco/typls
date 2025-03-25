// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: ["@nuxt/ui", "@vueuse/nuxt"],

  ui: {
    icons: ["tabler"],
  },

  // (optional) Enable the Nuxt devtools
  devtools: { enabled: true },

  // Enable SSG
  ssr: false,

  vite: {
    // Better support for Tauri CLI output
    clearScreen: false,
    // Enable environment variables
    // Additional environment variables can be found at
    // https://tauri.app/2/reference/environment-variables/
    envPrefix: ["VITE_", "TAURI_"],
    server: {
      // Tauri requires a consistent port
      strictPort: true,

      hmr: {
        // Use websocket for mobile hot reloading
        protocol: "ws",
        // Make sure it's available on the network
        host: "0.0.0.0",
        // Use a specific port for hmr
        port: 5183,
      },
    },
  },

  srcDir: "src",
  compatibilityDate: "2025-03-21",
});