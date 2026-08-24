<template>
  <aside class="rail">
    <h2>关键变量</h2>
    <button
      v-for="item in items"
      :key="item.key"
      type="button"
      class="kpi"
      :class="[item.status, { pinned: item.pinned, focus: item.focus }]"
      :title="item.hint"
      @click="onClick($event, item.key)"
    >
      <svg class="bar" viewBox="0 0 18 92" aria-hidden="true">
        <AnalogBar
          :width="18"
          :height="92"
          :value="item.value"
          :scale-lo="item.scaleLo"
          :scale-hi="item.scaleHi"
          :lo-op="item.loOp"
          :hi-op="item.hiOp"
          :setpoint="item.setpoint"
          :status="item.status"
        />
      </svg>
      <span class="copy">
        <span class="name">{{ item.name }}</span>
        <b class="pv">{{ item.text }}</b>
        <span class="unit">{{ item.unit }}</span>
      </span>
    </button>
  </aside>
</template>

<script>
import AnalogBar from "./AnalogBar.vue";

export default {
  components: { AnalogBar },
  props: {
    items: { type: Array, default: () => [] },
  },
  emits: ["pin"],
  methods: {
    onClick(ev, key) {
      this.$emit("pin", { key, shift: ev.shiftKey });
    },
  },
};
</script>

<style scoped>
.rail {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  padding: 0.45rem 0.4rem 0.6rem;
  background: var(--panel);
  border-left: 1px solid var(--rule);
  min-height: 0;
  overflow: auto;
  height: 100%;
  flex: 1;
}
h2 {
  margin: 0 0 0.2rem;
  font-size: 0.68rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-soft);
}
.kpi {
  display: grid;
  grid-template-columns: 18px 1fr;
  gap: 0.45rem;
  align-items: stretch;
  margin: 0;
  padding: 0.28rem 0.3rem;
  border: 1px solid transparent;
  background: transparent;
  color: var(--ink);
  text-align: left;
  cursor: pointer;
  flex: 1 1 0;
  min-height: 0;
}
.kpi:hover,
.kpi.focus {
  border-color: var(--rule);
  background: #eceef0;
}
.kpi.pinned {
  box-shadow: inset 2px 0 0 var(--event);
}
.kpi.advisory .pv {
  color: var(--advisory);
}
.kpi.trip .pv {
  color: var(--trip);
}
.bar {
  width: 18px;
  height: 100%;
  min-height: 48px;
  max-height: 92px;
  display: block;
  align-self: stretch;
}
.copy {
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  min-width: 0;
}
.name {
  font-size: 0.72rem;
  color: var(--ink-soft);
}
.pv {
  font-family: var(--type-data);
  font-size: 1.05rem;
  font-weight: 600;
  line-height: 1.15;
}
.unit {
  font-size: 0.68rem;
  color: var(--ink-soft);
}
.kpi:focus-visible {
  outline: 2px solid var(--ink);
  outline-offset: 1px;
}
</style>
