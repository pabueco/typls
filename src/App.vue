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
  trigger: {
    string: string;
  };
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

const duplicates = computed(() => {
  const abbrs = settings.value.expanders.map((e) => e.abbr);
  return abbrs.filter((a, i) => abbrs.indexOf(a) !== i);
});
</script>

<template>
  <div class="p-6 space-y-6">
    <h5 class="font-bold text-base">Trigger</h5>

    <UInput v-model="settings.trigger.string" />

    <h5 class="font-bold text-base">Expansions</h5>
    <div class="space-y-5">
      <div v-for="(expander, i) of settings.expanders" :key="`${i}`">
        <Expansion
          v-model="settings.expanders[i]"
          :duplicate="duplicates.includes(expander.abbr)"
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
