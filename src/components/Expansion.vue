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

const expander = useVModel($props, "modelValue", emit);
</script>

<template>
  <div class="flex gap-2">
    <div>
      <UFormGroup :error="duplicate ? 'Duplicate abbreviation' : false">
        <UInput
          v-model="expander.abbr"
          placeholder="Abbreviation"
          class="font-mono"
        />
      </UFormGroup>
    </div>
    <div class="pt-1.5">
      <UIcon name="i-tabler-arrow-right" class="text-gray-600" />
    </div>
    <div class="flex-1">
      <!-- <UInput v-model="expander.text" placeholder="Full text" /> -->
      <UTextarea
        v-model="expander.text"
        autoresize
        :rows="1"
        placeholder="Expanded text"
      />
    </div>
    <div>
      <UButton
        @click="emit('remove')"
        color="gray"
        icon="i-tabler-trash"
      ></UButton>
    </div>
  </div>
</template>
