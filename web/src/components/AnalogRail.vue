<template>
  <aside class="rail">
    <button
      v-for="item in items"
      :key="item.key"
      type="button"
      class="kpi"
      :class="[item.status, { pinned: item.pinned, focus: item.focus }]"
      :title="item.hint"
      draggable="false"
      @pointerdown="onDown($event, item)"
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
  emits: ["drag-tag", "focus-tag"],
  data() {
    return { pending: null };
  },
  beforeUnmount() {
    this.clearPending();
  },
  methods: {
    onDown(ev, item) {
      if (ev.button != null && ev.button !== 0) return;
      this.clearPending();
      this.pending = { key: item.key, x: ev.clientX, y: ev.clientY, pinned: item.pinned };
      window.addEventListener("pointermove", this.onWinMove, true);
      window.addEventListener("pointerup", this.onWinUp, true);
      window.addEventListener("pointercancel", this.onWinUp, true);
    },
    onWinMove(ev) {
      const p = this.pending;
      if (!p) return;
      const dx = ev.clientX - p.x;
      const dy = ev.clientY - p.y;
      if (Math.hypot(dx, dy) < 8) return;
      const rail = this.$el;
      const r = rail.getBoundingClientRect();
      const inside = ev.clientX >= r.left && ev.clientX <= r.right && ev.clientY >= r.top && ev.clientY <= r.bottom;
      const canScroll = rail.scrollHeight > rail.clientHeight + 1;
      if (inside && canScroll && Math.abs(dy) > Math.abs(dx) * 1.2) {
        this.clearPending();
        return;
      }
      const key = p.key;
      this.clearPending();
      this.$emit("drag-tag", { key, x: ev.clientX, y: ev.clientY });
    },
    onWinUp() {
      const p = this.pending;
      this.clearPending();
      if (p?.pinned) this.$emit("focus-tag", p.key);
    },
    clearPending() {
      this.pending = null;
      window.removeEventListener("pointermove", this.onWinMove, true);
      window.removeEventListener("pointerup", this.onWinUp, true);
      window.removeEventListener("pointercancel", this.onWinUp, true);
    },
  },
};
</script>

<style scoped>
.rail {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  padding: 0.35rem 0.4rem 0.55rem;
  background: var(--panel);
  min-height: 0;
  overflow: auto;
  height: 100%;
  flex: 1;
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
  cursor: grab;
  flex: 1 1 0;
  min-height: 0;
  touch-action: pan-y;
  user-select: none;
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
