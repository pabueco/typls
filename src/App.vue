<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { isEqual, cloneDeep } from "lodash-es";
import {
  check as checkForUpdates,
  type Update,
} from "@tauri-apps/plugin-updater";
import { getVersion, getName } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-shell";

// The updater seems to break the app on MacOS and causes a virus alert on Windows, so it's disabled for now.
// TODO: Enable updates when the updater is fixed.
const AUTO_UPDATES_ENABLED = false;

const GITHUB_REPO_URL = "https://github.com/pabueco/typls";

useHead({
  htmlAttrs: {
    class: "bg-gray-950 text-white",
  },
});

const metadata = ref({
  name: await getName(),
  version: await getVersion(),
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
    auto: boolean;
  };
  variables: {
    separator: string;
  };
  expansions: {
    abbr: string;
    text: string;
  }[];
};

const initialSettings = ref(await invoke<Settings>("get_settings"));
const settings = ref<Settings>(cloneDeep(initialSettings.value));

const haveSettingsChanged = ref(false);
const hasJustSaved = autoResetRef(false, 3000);

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

const duplicates = computed(() => {
  const abbrs = settings.value.expansions.map((e) => e.abbr);
  return abbrs.filter((a, i) => abbrs.indexOf(a) !== i);
});

const isInvalid = computed(() => {
  return Object.values(settingsErrors.value).some((e) =>
    Object.values(e).some((v) => v !== undefined)
  );
});

type SettingsErrors = {
  trigger: {
    string: string | undefined;
  };
  confirm: {
    chars: string | undefined;
  };
  variables: {
    separator: string | undefined;
  };
};
const settingsErrors = computed<SettingsErrors>(() => {
  return {
    trigger: {
      string:
        settings.value.trigger.string === ""
          ? "Cannot be empty."
          : settings.value.trigger.string.length > 1
          ? "Cannot be longer than one character."
          : settings.value.confirm.chars.includes(settings.value.trigger.string)
          ? "Cannot be the same as a confirmation character."
          : settings.value.variables.separator === settings.value.trigger.string
          ? "Cannot be the same as the variable separator character."
          : undefined,
    },
    confirm: {
      chars:
        settings.value.confirm.chars.length === 0
          ? "Cannot be empty."
          : settings.value.confirm.chars.some((c) =>
              [
                settings.value.trigger.string,
                settings.value.variables.separator,
              ].includes(c)
            )
          ? "Cannot contain the trigger or variable separator characters."
          : undefined,
    },
    variables: {
      separator:
        settings.value.variables.separator === ""
          ? "Cannot be empty."
          : settings.value.variables.separator.length > 1
          ? "Cannot be longer than one character."
          : settings.value.confirm.chars.includes(
              settings.value.variables.separator
            )
          ? "Cannot be the same as a confirmation character."
          : settings.value.trigger.string === settings.value.variables.separator
          ? "Cannot be the same as the trigger character."
          : undefined,
    },
  };
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

const resetSettings = async () => {
  const defaultSettings = await invoke<Settings>("get_default_settings");
  settings.value = {
    ...defaultSettings,
    expansions: settings.value.expansions,
  };
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

const availableUpdate = ref<Update | null>(null);
const isInstallingUpdate = ref(false);

const checkForAvailableUpdates = async (notifyWhenUpToDate = false) => {
  let update: Update | null = null;

  try {
    update = await checkForUpdates();
  } catch (e) {
    console.error(e);
    toast.add({
      title: "Something went wrong",
      description: `An error occurred while checking for updates. Please try again later.`,
      icon: "i-tabler-alert-triangle",
      color: "red",
    });
    return;
  }

  availableUpdate.value = update;

  if (!update) {
    if (notifyWhenUpToDate) {
      toast.add({
        title: "Up to date",
        description: `You are running the latest version.`,
        icon: "i-tabler-circle-check",
      });
    }

    return;
  }

  if (!AUTO_UPDATES_ENABLED) {
    toast.add({
      title: "Update available",
      description: `Version ${update.version} is available. You are currently running version ${update.currentVersion}. Please download the latest version from the GitHub repository and install it manually.`,
      icon: "i-tabler-info-circle",
      timeout: 0,
      actions: [
        {
          label: "View on GitHub",
          color: "primary",
          async click() {
            open(`${GITHUB_REPO_URL}/releases/latest`);
          },
        },
      ],
    });
  } else {
    toast.add({
      title: "Update available",
      description: `Version ${update.version} is available. You are currently running version ${update.currentVersion}.`,
      icon: "i-tabler-info-circle",
      timeout: 0,
      actions: [
        {
          label: "Update now",
          color: "primary",
          async click() {
            isInstallingUpdate.value = true;
            await update?.downloadAndInstall();

            toast.add({
              title: "Update installed",
              description: `Version ${update?.version} has been installed. Restart the app to apply the changes.`,
              icon: "i-tabler-circle-check",
              timeout: 0,
              actions: [
                {
                  label: "Restart now",
                  color: "primary",
                  async click() {
                    await relaunch();
                  },
                },
              ],
            });
          },
        },
      ],
    });
  }
};
</script>

<template>
  <div class="flex flex-col min-h-screen" spellcheck="false">
    <UNotifications />

    <div
      class="sticky top-0 z-10 w-full bg-gray-950 border-b border-gray-700 grid grid-cols-3 items-center justify-between px-8 py-5"
    >
      <div class="flex gap-1">
        <UTooltip text="Open config directory">
          <UButton
            icon="i-tabler-folder-cog"
            @click="openSettingsFolder"
            variant="ghost"
            color="gray"
            size="sm"
          />
        </UTooltip>
        <UTooltip text="Open GitHub repository">
          <UButton
            to="https://github.com/pabueco/typls"
            target="_blank"
            icon="i-tabler-brand-github"
            variant="ghost"
            color="gray"
            size="sm"
          />
        </UTooltip>
        <UTooltip :text="AUTO_UPDATES_ENABLED ? 'Check for updates' : 'Updates are currently disabled'">
          <UChip :show="!!availableUpdate">
            <UButton
              icon="i-tabler-cloud-search"
              @click="checkForAvailableUpdates(true)"
              variant="ghost"
              color="gray"
              size="sm"
              :disabled="!AUTO_UPDATES_ENABLED || isInstallingUpdate"
            />
          </UChip>
        </UTooltip>
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
      </div>
    </div>

    <div class="p-8 space-y-10">
      <div>
        <div class="flex mb-5 items-center justify-between">
          <div class="flex items-center gap-3">
            <h5 class="font-bold text-xl">Settings</h5>

            <UTooltip text="Reset settings to default">
              <UButton
                icon="i-tabler-arrow-back-up"
                @click="resetSettings"
                variant="ghost"
                color="gray"
                size="sm"
              />
            </UTooltip>
          </div>

          <div class="flex space-x-2 text-sm items-center">
            <div class="text-gray-500">Example:</div>
            <div
              class="font-mono rounded bg-gray-800 px-1.5 py-0.5 text-gray-200 whitespace-pre"
            >
              {{ settings.trigger.string }}hi{{
                settings.variables.separator
              }}foo{{ settings.variables.separator }}name=bar{{
                settings.confirm.chars[0]
              }}
            </div>
          </div>
        </div>

        <div
          class="grid gap-6"
          :style="{
            gridTemplateColumns: '1fr 1fr 300px',
          }"
        >
          <UFormGroup
            label="Trigger character"
            :error="settingsErrors.trigger.string"
          >
            <UInput
              v-model="settings.trigger.string"
              placeholder="Character"
              maxlength="1"
              class="font-mono"
              @click="(e: MouseEvent) => (e.target as HTMLInputElement).select()"
            />

            <template #hint>
              <UTooltip :ui="{ base: '!h-auto' }">
                <UIcon name="i-tabler-help" />

                <template #text>
                  Indicates the start of an <br />
                  abbreviation when tryping.
                </template>
              </UTooltip>
            </template>
          </UFormGroup>

          <UFormGroup
            label="Variable separator"
            :error="settingsErrors.variables.separator"
          >
            <UInput
              v-model="settings.variables.separator"
              placeholder="Character"
              maxlength="1"
              class="font-mono"
              @click="(e: MouseEvent) => (e.target as HTMLInputElement).select()"
            />

            <template #hint>
              <UTooltip :ui="{ base: '!h-auto' }">
                <UIcon name="i-tabler-help" />

                <template #text>
                  Used for appending variables <br />
                  to an abbreviation.
                </template>
              </UTooltip>
            </template>
          </UFormGroup>

          <div class="">
            <div class="flex items-start">
              <div class="flex-1">
                <UFormGroup
                  label="Confirmation characters & keys"
                  :error="settingsErrors.confirm.chars"
                >
                  <UInput
                    :model-value="settings.confirm.chars.join('')"
                    @update:model-value="
                      settings.confirm.chars = $event.split('')
                    "
                    placeholder="Character"
                    class="font-mono"
                  />
                </UFormGroup>
              </div>

              <div class="ml-2">
                <div class="flex justify-end">
                  <UTooltip :ui="{ base: '!h-auto' }">
                    <UIcon name="i-tabler-help" class="text-sm text-gray-400" />

                    <template #text>
                      Indicates that the typed <br />
                      abbreviation should be expanded.
                    </template>
                  </UTooltip>
                </div>

                <div class="flex mt-2.5 gap-2">
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
                      :variant="
                        settings.confirm.keyRightArrow ? 'soft' : 'ghost'
                      "
                      :color="
                        settings.confirm.keyRightArrow ? 'primary' : 'gray'
                      "
                    />
                  </UTooltip>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div class="mt-6 space-y-2">
          <UFormGroup>
            <UCheckbox
              v-model="settings.confirm.append"
              label="Append confirmation characters/keys to the expanded text."
            />
          </UFormGroup>
          <UFormGroup>
            <UCheckbox
              v-model="settings.confirm.auto"
              label="Automatically expand abbreviation while typing, when there are no variables in the text."
            />
          </UFormGroup>
        </div>
      </div>

      <div class="border-b border-gray-900"></div>

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
              :invalid-chars="settings.confirm.chars"
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
    </div>

    <div class="mt-auto p-8 pt-0 text-xs text-gray-500">
      <div class="border-b border-gray-900 mb-8"></div>

      <div class="w-full flex justify-center gap-1 font-mono">
        <span>{{ metadata.name }}</span>
        <a
          :href="`${GITHUB_REPO_URL}/releases/v${metadata.version}`"
          target="_blank"
          class="hover:text-white transition"
          >v{{ metadata.version }}</a
        >
        <span class="text-gray-700 mx-1 text-[10px]">•</span>
        <span>by</span>
        <a
          href="https://pabue.co"
          target="_blank"
          class="hover:text-white transition"
          >pabueco</a
        >
      </div>
    </div>
  </div>
</template>
