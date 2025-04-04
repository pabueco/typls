<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { confirm } from "@tauri-apps/plugin-dialog";
import { isEqual, cloneDeep } from "es-toolkit";
import {
  check as checkForUpdates,
  type Update,
} from "@tauri-apps/plugin-updater";
import { getVersion, getName } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-shell";
import { useSortable } from "@vueuse/integrations/useSortable";
import { UFormField } from "#components";
import { uniqueId } from "es-toolkit/compat";
import type { Group, Settings } from "./types";

// The updater seems to break the app on MacOS and causes a virus alert on Windows, so it's disabled for now.
// TODO: Enable updates when the updater is fixed.
const AUTO_UPDATES_ENABLED = false;

const GITHUB_REPO_URL = "https://github.com/pabueco/typls";

useHead({
  bodyAttrs: {
    class: "dark:bg-neutral-950 dark:text-white bg-neutral-50",
  },
});

const metadata = ref({
  name: await getName(),
  version: await getVersion(),
});

const sourceSettings = await invoke<Settings>("get_settings");

const initialSettings = ref<Settings>({
  ...sourceSettings,
  expansions: sourceSettings.expansions.map((e) => ({
    ...e,
    id: uniqueId("exp_"),
  })),
});
const settings = ref<Settings>(cloneDeep(initialSettings.value));

const isDragging = ref(false);

const expansionsListRef = useTemplateRef("expansionsListRef");

useSortable(expansionsListRef, settings.value.expansions, {
  handle: "[data-is-handle]",
  animation: 200,
  // Without this, every second drag attempt does not work on MacOS.
  supportPointer: false,
  forceFallback: true,
  onStart: () => (isDragging.value = true),
  onEnd: () => (isDragging.value = false),
});

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
  await invoke<Settings>("set_settings", {
    settings: {
      ...settings.value,
      expansions: settings.value.expansions.map((e) => ({
        ...e,
        id: undefined,
      })),
    },
  });
  if (options?.showToast) {
    toast.add({ title: "Saved", icon: "i-tabler-circle-check" });
  }
  hasJustSaved.value = true;
  initialSettings.value = cloneDeep(settings.value);
  haveSettingsChanged.value = false;
};

const duplicates = computed(() => {
  return settings.value.expansions.filter((e) => {
    return (
      settings.value.expansions.filter(
        (e2) => e2.abbr === e.abbr && e2.group === e.group
      ).length > 1
    );
  });
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

const addNewExpansion = (index = 0) => {
  settings.value.expansions.splice(index, 0, {
    id: uniqueId("exp_"),
    abbr: "",
    text: "",
  });
};

const addNewGroup = () => {
  settings.value.groups.push({
    id: crypto.randomUUID(),
    name: "(empty)",
    apps: [],
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
      color: "error",
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
      duration: 0,
      actions: [
        {
          label: "View on GitHub",
          color: "primary",
          async onClick() {
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
      duration: 0,
      actions: [
        {
          label: "Update now",
          color: "primary",
          async onClick() {
            isInstallingUpdate.value = true;
            await update?.downloadAndInstall();

            toast.add({
              title: "Update installed",
              description: `Version ${update?.version} has been installed. Restart the app to apply the changes.`,
              icon: "i-tabler-circle-check",
              duration: 0,
              actions: [
                {
                  label: "Restart now",
                  color: "primary",
                  async onClick() {
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

async function removeGroup(group: Group) {
  if (
    group.apps?.length &&
    !(await confirm("Delete group and remove from expansions?", {
      kind: "warning",
    }))
  )
    return;

  settings.value.groups?.splice(settings.value.groups.indexOf(group), 1);
  settings.value.expansions.forEach((e) => {
    if (e.group === group.id) {
      e.group = null;
    }
  });
}

onMounted(() => {
  // useTimeoutFn(() => checkForAvailableUpdates(false), 3000);
});
</script>

<template>
  <UApp>
    <div class="flex flex-col min-h-screen" spellcheck="false">
      <div
        class="sticky top-0 z-30 w-full dark:bg-neutral-950 border-b dark:border-neutral-800 bg-neutral-50 border-neutral-200 grid grid-cols-3 items-center justify-between px-8 py-5"
      >
        <div class="flex gap-1 items-center">
          <UTooltip text="Open config directory">
            <UButton
              icon="i-tabler-folder-cog"
              @click="openSettingsFolder"
              variant="ghost"
              color="neutral"
              size="sm"
            />
          </UTooltip>
          <UTooltip text="Open GitHub repository">
            <UButton
              to="https://github.com/pabueco/typls"
              target="_blank"
              icon="i-tabler-brand-github"
              variant="ghost"
              color="neutral"
              size="sm"
            />
          </UTooltip>
          <UTooltip
            :text="
              AUTO_UPDATES_ENABLED
                ? 'Check for updates'
                : 'Updates are currently disabled'
            "
          >
            <UChip :show="!!availableUpdate">
              <UButton
                icon="i-tabler-cloud-search"
                @click="checkForAvailableUpdates(true)"
                variant="ghost"
                color="neutral"
                size="sm"
                :disabled="!AUTO_UPDATES_ENABLED || isInstallingUpdate"
              />
            </UChip>
          </UTooltip>

          <div class="ml-3">
            <Transition
              enter-from-class="opacity-0 -translate-x-2"
              leave-to-class="opacity-0 -translate-x-2"
              enter-active-class="transition duration-200"
              leave-active-class="transition duration-200"
            >
              <div v-if="hasJustSaved" class="flex items-center gap-1.5">
                <UIcon
                  name="i-tabler-circle-check"
                  class="text-green-500 text-lg"
                />
                <span
                  class="text-xs bg:text-primary-300 text-primary-600 dark:text-primary-500 font-medium"
                  >Saved</span
                >
              </div>
            </Transition>
          </div>
        </div>
        <div class="flex items-center justify-center">
          <div class="font-bold text-xl font-mono">typls</div>
        </div>
        <div class="flex items-center justify-end">
          <USelectMenu
            v-model="settings.activeGroup"
            value-key="id"
            label-key="name"
            create-item
            :items="[{ name: 'Auto', id: null }, ...(settings.groups ?? [])]"
            class="w-36"
            placeholder="Auto"
            variant="soft"
            icon="i-tabler-stack"
          />
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
                  color="neutral"
                  size="sm"
                />
              </UTooltip>
            </div>

            <div class="flex space-x-2 text-sm items-center">
              <div class="text-neutral-500">Example:</div>
              <div
                class="font-mono rounded dark:bg-neutral-800 bg-neutral-200/65 px-1.5 py-0.5 dark:text-neutral-200 text-black whitespace-pre"
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
            <UFormField
              label="Trigger character"
              :error="settingsErrors.trigger.string"
            >
              <UInput
                v-model="settings.trigger.string"
                placeholder="Character"
                maxlength="1"
                class="font-mono w-full"
                @click="(e: MouseEvent) => (e.target as HTMLInputElement).select()"
              />

              <template #hint>
                <UTooltip :ui="{ content: '!h-auto' }">
                  <UIcon name="i-tabler-help" />

                  <template #content>
                    Indicates the start of an <br />
                    abbreviation when tryping.
                  </template>
                </UTooltip>
              </template>
            </UFormField>

            <UFormField
              label="Variable separator"
              :error="settingsErrors.variables.separator"
            >
              <UInput
                v-model="settings.variables.separator"
                placeholder="Character"
                maxlength="1"
                class="font-mono w-full"
                @click="(e: MouseEvent) => (e.target as HTMLInputElement).select()"
              />

              <template #hint>
                <UTooltip :ui="{ content: '!h-auto' }">
                  <UIcon name="i-tabler-help" />

                  <template #content>
                    Used for appending variables <br />
                    to an abbreviation.
                  </template>
                </UTooltip>
              </template>
            </UFormField>

            <div class="">
              <div class="flex items-start">
                <div class="flex-1">
                  <UFormField
                    label="Confirmation characters & keys"
                    :error="settingsErrors.confirm.chars"
                  >
                    <UInput
                      :model-value="settings.confirm.chars.join('')"
                      @update:model-value="
                        (v) => (settings.confirm.chars = v.toString().split(''))
                      "
                      placeholder="Character"
                      class="font-mono w-full"
                    />
                  </UFormField>
                </div>

                <div class="ml-2">
                  <div class="flex justify-end">
                    <UTooltip :ui="{ content: '!h-auto' }">
                      <UIcon
                        name="i-tabler-help"
                        class="text-sm text-neutral-400"
                      />

                      <template #content>
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
                        :color="
                          settings.confirm.keyEnter ? 'primary' : 'neutral'
                        "
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
                          settings.confirm.keyRightArrow ? 'primary' : 'neutral'
                        "
                      />
                    </UTooltip>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="mt-6 space-y-2">
            <UFormField>
              <UCheckbox
                v-model="settings.confirm.append"
                label="Append confirmation characters/keys to the expanded text."
              />
            </UFormField>
            <UFormField>
              <UCheckbox
                v-model="settings.confirm.auto"
                label="Automatically expand abbreviation while typing, when there are no variables in the text."
              />
            </UFormField>
          </div>
        </div>

        <div class="border-b dark:border-neutral-900 border-neutral-200"></div>

        <div class="space-y-6">
          <div class="flex justify-between items-center">
            <h5 class="font-bold text-2xl">Expansions</h5>

            <div class="flex items-center gap-2">
              <UInput
                v-model="searchQuery"
                placeholder="Type to search..."
                type="search"
                variant="none"
                icon="i-tabler-search"
              />
              <UButton
                @click="addNewExpansion(0)"
                variant="outline"
                icon="i-tabler-plus"
              >
                Add new
              </UButton>
            </div>
          </div>

          <div ref="expansionsListRef">
            <div
              v-for="(expansion, i) of settings.expansions"
              :key="expansion.id || i"
              class="relative"
            >
              <Expansion
                v-model="settings.expansions[i]"
                :duplicate="duplicates.includes(settings.expansions[i])"
                :invalid-chars="settings.confirm.chars"
                :groups="settings.groups"
                @remove="settings.expansions.splice(i, 1)"
                @create:group="
                  (g) => (settings.groups = [...(settings.groups ?? []), g])
                "
                class="mt-5 dark:border-neutral-900 border-neutral-200 pb-5"
                :class="{
                  hidden: !expansionsFiltered.includes(expansion),
                  'border-b': i !== settings.expansions.length - 1,
                  'pointer-events-none': isDragging,
                }"
              />
              <div
                v-if="i < settings.expansions.length - 1"
                class="absolute bottom-0 w-full left-1/2 -translate-x-1/2 flex justify-center items-center translate-y-1/2 hover:opacity-100 opacity-0 z-10 group transition hover:delay-300"
              >
                <div
                  class="scale-50 group-hover:scale-100 transition group-hover:delay-300"
                >
                  <UButton
                    size="xs"
                    color="primary"
                    icon="i-tabler-plus"
                    @click="addNewExpansion(i + 1)"
                  ></UButton>
                </div>
              </div>
            </div>
          </div>

          <div class="flex items-center justify-center">
            <UButton
              @click="addNewExpansion(settings.expansions.length)"
              variant="outline"
              icon="i-tabler-plus"
            >
              Add new expansion
            </UButton>
          </div>

          <div class="flex items-center justify-between gap-4 mt-16">
            <h5 class="font-bold text-2xl">Groups</h5>

            <div>
              <UButton
                @click="addNewGroup()"
                variant="outline"
                icon="i-tabler-plus"
                color="neutral"
              >
                Add new group
              </UButton>
            </div>
          </div>

          <div>
            <div v-for="(group, i) of settings.groups ?? []" :key="group.id">
              <Group
                v-model="settings.groups[i]"
                @remove="removeGroup(group)"
              />

              <div
                v-if="i < settings.groups.length - 1"
                class="dark:border-neutral-900 border-neutral-200 border my-5"
              ></div>
            </div>
          </div>
        </div>

        <div v-if="isInvalid" class="sticky inset-x-0 bottom-0 mt-10">
          <UAlert
            icon="i-tabler-alert-triangle-filled"
            color="error"
            variant="solid"
            title="Invalid settings!"
            description="Some of the settings are invalid. Please fix them before saving."
          />
        </div>
      </div>

      <div
        class="mt-auto p-8 pt-0 text-xs dark:text-neutral-500 text-neutral-400"
      >
        <div
          class="border-b dark:border-neutral-900 border-neutral-300 mb-8"
        ></div>

        <div class="w-full flex justify-center gap-1 font-mono">
          <span>{{ metadata.name }}</span>
          <a
            :href="`${GITHUB_REPO_URL}/releases/v${metadata.version}`"
            target="_blank"
            class="dark:hover:text-white hover:text-black transition"
            >v{{ metadata.version }}</a
          >
          <span class="dark:text-neutral-700 text-neutral-300 mx-1 text-[10px]"
            >•</span
          >
          <span>by</span>
          <a
            href="https://pabue.co"
            target="_blank"
            class="dark:hover:text-white hover:text-black transition"
            >pabueco</a
          >
        </div>
      </div>
    </div>
  </UApp>
</template>

<style>
.sortable-ghost {
  @apply opacity-35;
}
</style>
