<script setup lang="ts">
import { UFormField } from "#components";
import type { Expansion, Group } from "~/types";

const $props = defineProps<{
  modelValue: Partial<Expansion>;
  duplicate?: boolean;
  invalidChars?: string[];
  groups: Group[];
}>();

const emit = defineEmits<{
  (event: "update:modelValue", value: Partial<Expansion>): void;
  (event: "remove"): void;
  (event: "create:group", group: Group): void;
}>();

const expansion = useVModel($props, "modelValue", emit);

const isEditing = ref(!expansion.value.abbr && !expansion.value.text);

const error = computed(() => {
  if ($props.duplicate) return "Duplicate abbreviation";

  if (
    $props.invalidChars?.some((char) => expansion.value.abbr?.includes(char))
  ) {
    return `Contains confirm characters (${$props.invalidChars.join("")})`;
  }

  return false;
});

const showAsEditing = computed(() => {
  return isEditing.value || error.value;
});

const groupSelectOpen = ref(false);

function createGroupName(name: string) {
  const id = crypto.randomUUID();
  emit("create:group", {
    id,
    name,
    apps: [],
  });
  expansion.value.group = id;
}
</script>

<template>
  <div class="flex gap-2 group select-none" @keyup.escape="isEditing = false">
    <div
      data-is-handle
      class="group-hover:opacity-100 opacity-0 transition flex items-center"
    >
      <UButton
        color="neutral"
        icon="i-tabler-grip-vertical"
        variant="ghost"
        size="xs"
        class="opacity-50 hover:opacity-100 active:opacity-100 !cursor-grab"
      ></UButton>
    </div>
    <div class="w-full flex gap-2" @dblclick="isEditing = true">
      <div class="w-32" :class="{ 'pointer-events-none': !isEditing }">
        <UFormField :error="error">
          <UInput
            v-model="expansion.abbr"
            placeholder="Abbreviation"
            autoresizse
            class="font-mono text-right"
            :variant="showAsEditing ? 'outline' : 'none'"
            :disabled="!showAsEditing"
            :ui="{ base: '!cursor-auto text-right' }"
            autofocus
          />
        </UFormField>
      </div>
      <div class="pt-1.5">
        <UIcon name="i-tabler-arrow-right" class="text-neutral-500" />
      </div>
      <div class="flex-1" :class="{ 'pointer-events-none': !isEditing }">
        <UTextarea
          v-model="expansion.text"
          autoresize
          :rows="1"
          placeholder="Expanded text"
          :variant="isEditing ? 'outline' : 'none'"
          :disabled="!isEditing"
          :ui="{
            base: `!cursor-auto ${isEditing ? '!resize-y' : '!resize-none'}`,
          }"
          class="w-full"
        />
      </div>
    </div>
    <div class="flex gap-2 items-start">
      <div :class="{ 'pointer-events-none': !isEditing }">
        <USelectMenu
          v-model="expansion.group as any"
          value-key="id"
          label-key="name"
          create-item
          v-model:open="groupSelectOpen"
          :items="groups"
          class="w-36"
          @create="createGroupName"
          :placeholder="isEditing ? 'Add to group' : 'Global'"
          :ui="{
            trailing: expansion.group ? 'pe-1.5' : undefined,
          }"
          :variant="isEditing ? 'outline' : 'none'"
          :disabled="!isEditing"
        >
          <template #trailing>
            <span v-if="!isEditing"></span>
            <UButton
              v-else-if="!!expansion.group"
              size="xs"
              variant="ghost"
              icon="i-tabler-x"
              square
              color="neutral"
              @click.prevent.stop="delete expansion.group"
            />
          </template>
        </USelectMenu>
      </div>

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
