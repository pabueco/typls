<script setup lang="ts">
const $props = defineProps<{
  modelValue: {
    abbr: string;
    text: string;
  };
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
      <UInput v-model="expander.abbr" placeholder="Abbreviation" />
    </div>
    <div class="flex-1">
      <!-- <UInput v-model="expander.text" placeholder="Full text" /> -->
      <UTextarea v-model="expander.text" autoresize :rows="1" />
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
