<template>
  <div
    class="sash"
    :class="[axis, { on: armed }]"
    role="separator"
    :aria-orientation="axis === 'v' ? 'vertical' : 'horizontal'"
    :title="hint"
    @pointerdown="onDown"
    @pointermove="onMove"
    @pointerup="onUp"
    @pointercancel="onUp"
    @lostpointercapture="onUp"
    @dblclick="$emit('reset')"
  />
</template>

<script>
export default {
  props: {
    axis: { type: String, default: "v" },
    hint: { type: String, default: "拖动拉伸 · 双击还原" },
  },
  emits: ["start", "drag", "end", "reset"],
  data() {
    return { armed: false, x0: 0, y0: 0 };
  },
  methods: {
    onDown(ev) {
      if (ev.button != null && ev.button !== 0) return;
      ev.preventDefault();
      this.armed = true;
      this.x0 = ev.clientX;
      this.y0 = ev.clientY;
      try {
        ev.currentTarget.setPointerCapture(ev.pointerId);
      } catch {
        /* synthetic / test events may not support capture */
      }
      document.body.classList.add(this.axis === "v" ? "sash-col" : "sash-row");
      this.$emit("start", { x: ev.clientX, y: ev.clientY });
    },
    onMove(ev) {
      if (!this.armed) return;
      this.$emit("drag", {
        x: ev.clientX,
        y: ev.clientY,
        dx: ev.clientX - this.x0,
        dy: ev.clientY - this.y0,
      });
    },
    onUp() {
      if (!this.armed) return;
      this.armed = false;
      document.body.classList.remove("sash-col", "sash-row");
      this.$emit("end");
    },
  },
};
</script>

<style scoped>
.sash {
  position: relative;
  z-index: 4;
  flex: 0 0 auto;
  background: transparent;
  touch-action: none;
}
.sash.v {
  width: 5px;
  cursor: col-resize;
}
.sash.h {
  height: 5px;
  cursor: row-resize;
}
.sash::after {
  content: "";
  position: absolute;
}
.sash.v::after {
  top: 0;
  bottom: 0;
  left: -2px;
  right: -2px;
}
.sash.h::after {
  left: 0;
  right: 0;
  top: -2px;
  bottom: -2px;
}
.sash:hover,
.sash.on {
  background: var(--ink);
}
</style>
