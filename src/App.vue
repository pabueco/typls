<script lang="ts" setup>
import { show } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { isEqual, cloneDeep } from "lodash-es";

useHead({
  htmlAttrs: {
    class: "bg-gray-950 text-white",
  },
});

type Settings = {
  trigger: {
    string: string;
  };
  expansions: {
    abbr: string;
    text: string;
  }[];
};

const initialSettings = ref(await invoke<Settings>("get_settings"));
const settings = ref<Settings>(cloneDeep(initialSettings.value));

const haveSettingsChanged = ref(false);
const hasJustSaved = autoResetRef(false, 2000);

watchDebounced(
  settings,
  async (value) => {
    if (isInvalid.value) return;
    haveSettingsChanged.value = !isEqual(value, initialSettings);
    if (!haveSettingsChanged.value) return;
    console.log("settings changed", value);
    await save();
  },
  { debounce: 1000, deep: true }
);

const toast = useToast();

const save = async (options: { showToast: boolean } = { showToast: false }) => {
  await invoke("set_settings", { settings: settings.value });
  if (options?.showToast) {
    toast.add({ title: "Saved", icon: "i-tabler-circle-check" });
  }
  hasJustSaved.value = true;
  initialSettings.value = cloneDeep(settings.value);
  haveSettingsChanged.value = false;
};

const restart = async () => {
  await relaunch();
};

const duplicates = computed(() => {
  const abbrs = settings.value.expansions.map((e) => e.abbr);
  return abbrs.filter((a, i) => abbrs.indexOf(a) !== i);
});

const isInvalid = computed(() => {
  return (
    settings.value.trigger.string === "" ||
    settings.value.trigger.string.length > 1 ||
    duplicates.value.length > 0 ||
    settings.value.expansions
      .filter((e) => e.abbr || e.text)
      .some((e) => e.abbr === "")
  );
});

const searchQuery = ref("");

const expansionsFiltered = computed(() => {
  return settings.value.expansions.filter((e) => {
    return (
      e.abbr.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      e.text.toLowerCase().includes(searchQuery.value.toLowerCase())
    );
  });
});
</script>

<template>
  <div>
    <UNotifications />

    <div
      class="sticky top-0 z-10 w-full bg-gray-950 border-b border-gray-700 flex items-center justify-between px-6 py-4"
    >
      <div class="font-bold text-lg font-mono">typeless</div>
      <div class="flex items-center">
        <Transition
          enter-from-class="opacity-0 translate-x-2"
          leave-to-class="opacity-0 translate-x-2"
          enter-active-class="transition duration-200"
          leave-active-class="transition duration-200"
        >
          <div v-if="hasJustSaved" class="flex items-center gap-2">
            <UIcon
              name="i-tabler-circle-check"
              class="text-green-500 text-lg"
            />
            <span class="text-xs text-gray-300">Saved</span>
          </div>
        </Transition>
        <!-- <UButton
          @click="save({ showToast: true })"
          color="primary"
          variant="solid"
          :disabled="!haveSettingsChanged || isInvalid"
        >
          Save
        </UButton> -->
      </div>
    </div>

    <div class="p-6 space-y-8">
      <UFormGroup
        label="Trigger"
        :error="
          isInvalid ? 'Cannot be empty or longer than one character.' : ''
        "
        help="The character that indicates that the word after it should be expanded."
      >
        <UInput
          v-model="settings.trigger.string"
          placeholder="Character"
          maxlength="1"
          class="font-mono"
          @click="(e: MouseEvent) => (e.target as HTMLInputElement).select()"
        />
      </UFormGroup>

      <div class="flex justify-between items-center">
        <h5 class="font-bold text-xl">Expansions</h5>

        <div class="flex items-center gap-2">
          <UInput
            v-model="searchQuery"
            placeholder="Type to search..."
            type="search"
            variant="none"
            icon="i-tabler-search"
          />
          <UButton
            @click="settings.expansions.push({ abbr: '', text: '' })"
            variant="outline"
          >
            Add new
          </UButton>
        </div>
      </div>

      <div>
        <div v-for="(expansion, i) of settings.expansions" :key="`${i}`">
          <Expansion
            v-model="settings.expansions[i]"
            :duplicate="duplicates.includes(expansion.abbr)"
            @remove="settings.expansions.splice(i, 1)"
            class="mt-5 border-b border-gray-800 pb-5"
            :class="{ hidden: !expansionsFiltered.includes(expansion) }"
          />
        </div>
      </div>

      <div>
        <UButton
          @click="settings.expansions.push({ abbr: '', text: '' })"
          block
          color="gray"
        >
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
