<script setup lang="ts">
const $props = defineProps<{
  modelValue: {
    abbr: string;
    text: string;
  };
  duplicate?: boolean;
}>();

const emit = defineEmits<{
  (
    event: "update:modelValue",
    value: {
      abbr: string;
      text: string;
    }
  ): void;
  (event: "remove"): void;
}>();

const expansion = useVModel($props, "modelValue", emit);

const isEditing = ref(!expansion.value.abbr && !expansion.value.text);
</script>

<template>
  <div class="flex gap-2">
    <div class="contents" @dblclick="isEditing = true">
      <div class="w-32">
        <UFormGroup :error="duplicate ? 'Duplicate abbreviation' : false">
          <UInput
            v-model="expansion.abbr"
            placeholder="Abbreviation"
            class="font-mono text-right"
            :variant="isEditing ? 'outline' : 'none'"
            :disabled="!isEditing"
            :ui="{ base: '!cursor-auto text-right' }"
            autofocus
          />
        </UFormGroup>
      </div>
      <div class="pt-1.5">
        <UIcon name="i-tabler-arrow-right" class="text-gray-500" />
      </div>
      <div class="flex-1">
        <UTextarea
          v-model="expansion.text"
          autoresize
          :rows="1"
          placeholder="Expanded text"
          :variant="isEditing ? 'outline' : 'none'"
          :disabled="!isEditing"
          :ui="{ base: '!cursor-auto' }"
        />
      </div>
    </div>
    <div class="flex gap-2 items-start">
      <UButton
        @click="emit('remove')"
        color="red"
        icon="i-tabler-trash"
        :class="{
          invisible: !isEditing,
        }"
      ></UButton>
      <UButton
        @click="isEditing = !isEditing"
        color="gray"
        :icon="isEditing ? 'i-tabler-check' : 'i-tabler-pencil'"
        variant="ghost"
      ></UButton>
    </div>
  </div>
</template>
