<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { isEqual } from "lodash-es";

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

const initialSettings = await invoke<Settings>("get_settings");
const settings = ref<Settings>(structuredClone(initialSettings));

const haveSettingsChanged = ref(false);

watchDebounced(
  settings,
  async (value) => {
    haveSettingsChanged.value = !isEqual(value, initialSettings);
    if (!haveSettingsChanged.value) return;
    console.log("settings changed", value);
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

    <div v-if="haveSettingsChanged">
      <div>A restart is required for changes to take effect.</div>
      <UButton @click="restart">Restart</UButton>
    </div>
  </div>
</template>
