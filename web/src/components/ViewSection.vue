<template>
  <section class="view" :class="{ open: modelValue, nested }">
    <button
      type="button"
      class="view-head"
      :aria-expanded="modelValue"
      @click="$emit('update:modelValue', !modelValue)"
    >
      <i class="chev" aria-hidden="true" />
      <span>{{ title }}</span>
    </button>
    <div v-show="modelValue" class="view-body">
      <slot />
    </div>
  </section>
</template>

<script>
export default {
  props: {
    modelValue: { type: Boolean, default: true },
    title: { type: String, required: true },
    nested: { type: Boolean, default: false },
  },
  emits: ["update:modelValue"],
};
</script>

<style scoped>
.view {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 22px;
  overflow: hidden;
}
.view-head {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  flex: 0 0 22px;
  margin: 0;
  padding: 0 0.45rem;
  border: 0;
  border-bottom: 1px solid var(--rule);
  background: #dfe2e5;
  color: var(--ink-soft);
  font: inherit;
  font-size: 0.68rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  text-align: left;
  cursor: pointer;
  user-select: none;
}
.view-head:hover {
  background: #d4d8dc;
  color: var(--ink);
}
.nested .view-head {
  background: transparent;
  border-bottom: 0;
  letter-spacing: 0;
  text-transform: none;
  font-size: 0.72rem;
  color: var(--ink);
  padding: 0.1rem 0.35rem 0.1rem 0.15rem;
}
.chev {
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 4px 0 4px 6px;
  border-color: transparent transparent transparent currentColor;
  transform: rotate(0deg);
  transition: transform 0.12s ease;
}
.open > .view-head > .chev {
  transform: rotate(90deg);
}
.view-body {
  flex: 1 1 auto;
  min-height: 0;
  overflow: auto;
  padding: 0.45rem 0.55rem 0.6rem;
}
.nested .view-body {
  padding: 0 0 0.25rem 0.7rem;
}
@media (prefers-reduced-motion: reduce) {
  .chev {
    transition: none;
  }
}
</style>
