<script lang="ts" setup>
import { show } from "@tauri-apps/api/app";
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
    if (isInvalid.value) return;
    haveSettingsChanged.value = !isEqual(value, initialSettings);
    if (!haveSettingsChanged.value) return;
    console.log("settings changed", value);
    await save();
  },
  { debounce: 500, maxWait: 1000, deep: true }
);

const toast = useToast();

const save = async (options: { showToast: boolean } = { showToast: false }) => {
  await invoke("set_settings", { settings: settings.value });
  if (options?.showToast) {
    toast.add({ title: "Saved", icon: "i-tabler-circle-check" });
  }
};

const restart = async () => {
  await relaunch();
};

const duplicates = computed(() => {
  const abbrs = settings.value.expanders.map((e) => e.abbr);
  return abbrs.filter((a, i) => abbrs.indexOf(a) !== i);
});

const isInvalid = computed(() => {
  return (
    settings.value.trigger.string === "" ||
    settings.value.trigger.string.length > 1 ||
    duplicates.value.length > 0 ||
    settings.value.expanders.some((e) => e.abbr === "")
  );
});
</script>

<template>
  <div>
    <UNotifications />

    <div
      class="sticky top-0 z-10 w-full bg-gray-950 border-b border-gray-700 flex items-center justify-between px-6 py-4"
    >
      <div class="font-bold text-lg font-mono">typeless</div>
      <div>
        <UButton
          @click="save({ showToast: true })"
          color="primary"
          variant="solid"
          :disabled="!haveSettingsChanged || isInvalid"
        >
          Save
        </UButton>
      </div>
    </div>

    <div class="p-6 space-y-6">
      <UFormGroup
        label="Trigger"
        :error="
          isInvalid ? 'Cannot be empty or longer than one character.' : ''
        "
      >
        <UInput
          v-model="settings.trigger.string"
          placeholder="Character"
          maxlength="1"
        />
      </UFormGroup>

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

      <div v-if="isInvalid" class="sticky inset-x-0 bottom-0 mt-10">
        <UAlert
          icon="i-tabler-alert-triangle-filled"
          color="red"
          variant="solid"
          title="Invalid settings!"
          description="Some of the settings are invalid. Please fix them before saving."
        />
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
  </div>
</template>
