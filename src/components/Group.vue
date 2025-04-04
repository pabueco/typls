<script setup lang="ts">
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type { Group } from "../types";
import { last } from "es-toolkit";

const $props = defineProps<{
  modelValue: Partial<Group>;
}>();

const emit = defineEmits<{
  (event: "update:modelValue", value: Group): void;
  (event: "remove"): void;
}>();

const group = useVModel($props, "modelValue", emit);

async function selectApps() {
  const files = await openDialog({
    multiple: true,

    directory: false,
  });

  if (!files) return;

  if (!group.value.apps) {
    group.value.apps = [];
  }

  group.value.apps.push(...(files.filter(Boolean) as string[]));
}

const isEditing = ref(false);

function getAppName(app: string) {
  return last(app.split("/"))?.split(".")[0];
}

function removeApp(app: Group["apps"][number]) {
  group.value.apps?.splice(group.value.apps.indexOf(app), 1);
}
</script>

<template>
  <div class="flex items-center gap-5" @keyup.escape="isEditing = false">
    <div @dblclick="isEditing = true">
      <UInput
        v-model="group.name"
        :variant="isEditing ? 'outline' : 'none'"
        :disabled="!isEditing"
        :class="{ 'pointer-events-none': !isEditing }"
        class="w-45"
      />
    </div>

    <div class="flex flex-wrap gap-2">
      <div v-for="app of group.apps">
        <UTooltip :text="app">
          <UBadge color="neutral" variant="subtle" size="md">
            {{ getAppName(app) }}

            <template v-if="isEditing" #trailing>
              <UIcon name="i-tabler-x" @click="removeApp(app)" />
            </template>
          </UBadge>
        </UTooltip>
      </div>
      <UButton
        @click="selectApps()"
        size="xs"
        icon="i-tabler-plus"
        variant="ghost"
        color="neutral"
        :square="!!group.apps?.length"
        >{{ group.apps?.length ? "" : "Add apps" }}</UButton
      >
    </div>

    <div class="ml-auto flex gap-2">
      <UButton
        @click="emit('remove')"
        color="error"
        icon="i-tabler-trash"
        :class="{
          invisible: !isEditing,
        }"
      ></UButton>
      <UButton
        @click="isEditing = !isEditing"
        color="neutral"
        :icon="isEditing ? 'i-tabler-pencil-off' : 'i-tabler-pencil'"
        variant="ghost"
      ></UButton>
    </div>
  </div>
</template>
