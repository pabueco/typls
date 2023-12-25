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
  <div class="p-6 space-y-6">
    <UInput v-model="settings.triggerChar" />

    <div class="space-y-3">
      <div v-for="(expander, i) of settings.expanders" :key="`${i}`">
        <Expansion
          v-model="settings.expanders[i]"
          @remove="settings.expanders.splice(i, 1)"
        />
      </div>
    </div>

    <div>
      <UButton @click="settings.expanders.push({ abbr: '', text: '' })">
        Add new
      </UButton>
    </div>

    <!-- <div v-if="haveSettingsChanged">
      <UAlert
        icon="i-tabler-alert-triangle-filled"
        color="amber"
        variant="subtle"
        title="Restart required to apply changes"
      >
        <template #description>
          <p>
            The changes will take effect after a restart. Click the restart
            button to restart the app.
          </p>
          <div class="flex mt-3">
            <UButton @click="restart" color="amber" variant="solid"
              >Restart now</UButton
            >
          </div>
        </template>
      </UAlert>
    </div> -->
  </div>
</template>
