<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";

useHead({
  htmlAttrs: {
    class: "bg-gray-950 text-white",
  },
});

type Settings = {
  triggerChar: string;
  expanders: {
    abbr: string;
    text: string;
  }[];
};

const settings = ref<Settings>(await invoke("get_settings"));

watchDebounced(
  settings,
  async () => {
    console.log("changed!");
    await invoke("set_settings", { settings: settings.value });
  },
  { debounce: 500, maxWait: 1000, deep: true }
);

const restart = async () => {
  await relaunch();
};
</script>

<template>
  <div>
    <pre>
      {{ settings }}
    </pre>
    <UInput v-model="settings.triggerChar" />
    <div>A restart is required for changes to take effect.</div>
    <UButton @click="restart">Restart</UButton>
  </div>
</template>
