<template>
  <g class="abar">
    <rect class="track" x="0" y="0" :width="width" :height="height" />
    <rect
      v-if="band"
      class="op"
      :x="band.x"
      :y="band.y"
      :width="band.w"
      :height="band.h"
    />
    <rect class="fill" :class="status" v-bind="fillBox" />
    <line v-if="spLine" class="sp" v-bind="spLine" />
  </g>
</template>

<script>
import { fracOnScale } from "../limits.js";

export default {
  props: {
    width: { type: Number, default: 10 },
    height: { type: Number, default: 44 },
    vertical: { type: Boolean, default: true },
    value: { type: Number, default: null },
    scaleLo: { type: Number, default: 0 },
    scaleHi: { type: Number, default: 100 },
    loOp: { type: Number, default: null },
    hiOp: { type: Number, default: null },
    setpoint: { type: Number, default: null },
    status: { type: String, default: "normal" },
  },
  computed: {
    inner() {
      return this.vertical ? this.height - 2 : this.width - 2;
    },
    fillBox() {
      const f = fracOnScale(this.value, this.scaleLo, this.scaleHi);
      if (this.vertical) {
        const h = f * this.inner;
        return { x: 1, y: this.height - 1 - h, width: this.width - 2, height: h };
      }
      return { x: 1, y: 1, width: f * this.inner, height: this.height - 2 };
    },
    band() {
      if (this.loOp == null && this.hiOp == null) return null;
      const a = fracOnScale(this.loOp ?? this.scaleLo, this.scaleLo, this.scaleHi);
      const b = fracOnScale(this.hiOp ?? this.scaleHi, this.scaleLo, this.scaleHi);
      const lo = Math.min(a, b);
      const hi = Math.max(a, b);
      if (this.vertical) {
        const y1 = this.height - 1 - hi * this.inner;
        const y0 = this.height - 1 - lo * this.inner;
        return { x: 1, y: y1, w: this.width - 2, h: Math.max(1, y0 - y1) };
      }
      return { x: 1 + lo * this.inner, y: 1, w: Math.max(1, (hi - lo) * this.inner), h: this.height - 2 };
    },
    spLine() {
      if (this.setpoint == null || Number.isNaN(this.setpoint)) return null;
      const f = fracOnScale(this.setpoint, this.scaleLo, this.scaleHi);
      if (this.vertical) {
        const y = this.height - 1 - f * this.inner;
        return { x1: 0, y1: y, x2: this.width, y2: y };
      }
      const x = 1 + f * this.inner;
      return { x1: x, y1: 0, x2: x, y2: this.height };
    },
  },
};
</script>

<style scoped>
.track {
  fill: #c9ced2;
  stroke: #8b9298;
  stroke-width: 0.8;
}
.op {
  fill: #b8c0c6;
  opacity: 0.55;
}
.fill.normal {
  fill: #2c3338;
  opacity: 0.38;
}
.fill.advisory {
  fill: var(--advisory);
  opacity: 0.95;
}
.fill.trip {
  fill: var(--trip);
  opacity: 0.95;
}
.sp {
  stroke: var(--ink);
  stroke-width: 1.4;
}
</style>
