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
  confirm: {
    chars: string[];
    keyEnter: boolean;
    keyRightArrow: boolean;
    append: boolean;
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

watch(
  settings,
  () => {
    if (hasJustSaved.value) {
      hasJustSaved.value = false;
    }
  },
  { deep: true }
);

watchDebounced(
  settings,
  async (value) => {
    if (isInvalid.value) return;
    haveSettingsChanged.value = !isEqual(value, initialSettings.value);
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

const resetConfirmChars = async () => {
  const defaultSettings = await invoke<Settings>("get_default_settings");
  settings.value.confirm.chars = defaultSettings.confirm.chars;
};

const openSettingsFolder = async () => {
  await invoke("open_settings_dir");
};

const addNewExpansion = (above = false) => {
  settings.value.expansions[above ? "unshift" : "push"]({
    abbr: "",
    text: "",
  });
};
</script>

<template>
  <div>
    <UNotifications />

    <div
      class="sticky top-0 z-10 w-full bg-gray-950 border-b border-gray-700 grid grid-cols-3 items-center justify-between px-8 py-5"
    >
      <div class="flex gap-1">
        <UTooltip text="Open settings directory">
          <UButton
            icon="i-tabler-folder-cog"
            @click="openSettingsFolder"
            variant="ghost"
            color="gray"
            size="sm"
          />
        </UTooltip>
        <UButton
          to="https://github.com/pabueco"
          target="_blank"
          icon="i-tabler-brand-github"
          variant="ghost"
          color="gray"
          size="sm"
        />
      </div>
      <div class="flex items-center justify-center">
        <div class="font-bold text-xl font-mono">typls</div>
      </div>
      <div class="flex items-center justify-end">
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

    <div class="p-8 space-y-10">
      <div class="grid grid-cols-2 gap-6">
        <div>
          <UFormGroup
            label="Trigger"
            :error="
              isInvalid ? 'Cannot be empty or longer than one character.' : ''
            "
            help="The character that starts the capturing."
          >
            <UInput
              v-model="settings.trigger.string"
              placeholder="Character"
              maxlength="1"
              class="font-mono"
              @click="(e: MouseEvent) => (e.target as HTMLInputElement).select()"
            />
          </UFormGroup>
        </div>

        <div class="">
          <div class="flex items-start">
            <div class="flex-1">
              <UFormGroup
                label="Confirmation"
                :error="
                  isInvalid
                    ? 'Cannot be empty or longer than one character.'
                    : ''
                "
                help="Characters expanding the captured text."
              >
                <UInput
                  :model-value="settings.confirm.chars.join('')"
                  @update:model-value="
                    settings.confirm.chars = $event.split('')
                  "
                  placeholder="Character"
                  class="font-mono"
                />

                <template #hint>
                  <button
                    @click="resetConfirmChars"
                    class="text-xs hover:text-white transition"
                  >
                    Reset
                  </button>
                </template>
              </UFormGroup>
            </div>

            <div class="flex mt-6 ml-2 gap-2">
              <UTooltip text="Enter key">
                <UButton
                  icon="i-tabler-corner-down-left"
                  @click="
                    settings.confirm.keyEnter = !settings.confirm.keyEnter
                  "
                  :variant="settings.confirm.keyEnter ? 'soft' : 'ghost'"
                  :color="settings.confirm.keyEnter ? 'primary' : 'gray'"
                />
              </UTooltip>
              <UTooltip text="Right arrow key">
                <UButton
                  icon="i-tabler-arrow-right"
                  @click="
                    settings.confirm.keyRightArrow =
                      !settings.confirm.keyRightArrow
                  "
                  :variant="settings.confirm.keyRightArrow ? 'soft' : 'ghost'"
                  :color="settings.confirm.keyRightArrow ? 'primary' : 'gray'"
                />
              </UTooltip>
            </div>
          </div>
          <div class="mt-3">
            <UFormGroup>
              <UCheckbox
                v-model="settings.confirm.append"
                label="Append the characters/keys to the expanded text."
              />
            </UFormGroup>
          </div>
        </div>
      </div>

      <div class="space-y-6">
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
            <UButton @click="addNewExpansion(true)" variant="outline">
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
              class="mt-5 border-gray-800 pb-5"
              :class="{
                hidden: !expansionsFiltered.includes(expansion),
                'border-b': i !== settings.expansions.length - 1,
              }"
            />
          </div>
        </div>

        <div>
          <UButton @click="addNewExpansion(false)" block color="gray">
            Add new expansion
          </UButton>
        </div>
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
