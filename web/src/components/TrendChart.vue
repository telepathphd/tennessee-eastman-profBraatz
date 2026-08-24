<template>
  <div class="chart-wrap">
    <canvas ref="canvas" @pointerdown="onPointer" @pointermove="onDrag" />
    <div v-if="!series.length" class="empty">选择下方标签，运行仿真后在记录纸上出线。</div>
  </div>
</template>

<script>
const PENS = ["#1a5f8a", "#9a1f2e", "#2f6b4f", "#c47a12", "#5b3d8f", "#3b5368"];

export default {
  props: {
    timeS: { type: Array, default: () => [] },
    series: { type: Array, default: () => [] },
    cursor: { type: Number, default: 0 },
    injections: { type: Array, default: () => [] },
    shutdownTimeS: { type: Number, default: null },
  },
  emits: ["update:cursor"],
  mounted() {
    this.ro = new ResizeObserver(() => this.draw());
    this.ro.observe(this.$refs.canvas);
    this.draw();
  },
  beforeUnmount() {
    this.ro?.disconnect();
  },
  watch: {
    timeS: "draw",
    series: { handler: "draw", deep: true },
    cursor: "draw",
    injections: "draw",
    shutdownTimeS: "draw",
  },
  methods: {
    draw() {
      const canvas = this.$refs.canvas;
      if (!canvas) return;
      const dpr = window.devicePixelRatio || 1;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight || 260;
      canvas.width = Math.max(1, Math.floor(w * dpr));
      canvas.height = Math.max(1, Math.floor(h * dpr));
      const ctx = canvas.getContext("2d");
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.fillStyle = "#edf3ea";
      ctx.fillRect(0, 0, w, h);

      const pad = { l: 48, r: 16, t: 12, b: 28 };
      const iw = w - pad.l - pad.r;
      const ih = h - pad.t - pad.b;

      ctx.strokeStyle = "#b7c9b0";
      ctx.lineWidth = 1;
      for (let i = 0; i <= 8; i++) {
        const y = pad.t + (ih * i) / 8;
        ctx.beginPath();
        ctx.moveTo(pad.l, y);
        ctx.lineTo(pad.l + iw, y);
        ctx.stroke();
      }
      for (let i = 0; i <= 12; i++) {
        const x = pad.l + (iw * i) / 12;
        ctx.beginPath();
        ctx.moveTo(x, pad.t);
        ctx.lineTo(x, pad.t + ih);
        ctx.stroke();
      }

      const n = this.timeS.length;
      if (n < 2 || !this.series.length) return;
      const t0 = this.timeS[0];
      const t1 = this.timeS[n - 1];
      const span = Math.max(1, t1 - t0);
      const xOf = (t) => pad.l + ((t - t0) / span) * iw;

      for (const inj of this.injections) {
        const x = xOf(inj.start_step);
        ctx.strokeStyle = "#c47a12";
        ctx.setLineDash([4, 3]);
        ctx.beginPath();
        ctx.moveTo(x, pad.t);
        ctx.lineTo(x, pad.t + ih);
        ctx.stroke();
        ctx.setLineDash([]);
      }
      if (this.shutdownTimeS != null) {
        const x = xOf(this.shutdownTimeS);
        ctx.strokeStyle = "#9a1f2e";
        ctx.beginPath();
        ctx.moveTo(x, pad.t);
        ctx.lineTo(x, pad.t + ih);
        ctx.stroke();
      }

      this.series.forEach((s, si) => {
        const vals = s.values;
        let lo = Infinity;
        let hi = -Infinity;
        for (const v of vals) {
          if (v < lo) lo = v;
          if (v > hi) hi = v;
        }
        if (!Number.isFinite(lo)) return;
        if (hi === lo) {
          lo -= 1;
          hi += 1;
        }
        ctx.strokeStyle = PENS[si % PENS.length];
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        for (let i = 0; i < n; i++) {
          const x = xOf(this.timeS[i]);
          const y = pad.t + (1 - (vals[i] - lo) / (hi - lo)) * ih;
          if (i === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        }
        ctx.stroke();
      });

      const cx = xOf(this.timeS[Math.min(this.cursor, n - 1)] ?? t0);
      ctx.strokeStyle = "#16324f";
      ctx.lineWidth = 1.2;
      ctx.beginPath();
      ctx.moveTo(cx, pad.t);
      ctx.lineTo(cx, pad.t + ih);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(cx, pad.t + ih);
      ctx.lineTo(cx - 5, pad.t + ih + 8);
      ctx.lineTo(cx + 5, pad.t + ih + 8);
      ctx.closePath();
      ctx.fillStyle = "#16324f";
      ctx.fill();

      ctx.fillStyle = "#3b5368";
      ctx.font = "11px ui-monospace, Consolas, monospace";
      ctx.fillText(this.fmtH(t0), pad.l, h - 8);
      ctx.textAlign = "right";
      ctx.fillText(this.fmtH(t1), pad.l + iw, h - 8);
      ctx.textAlign = "left";
    },
    fmtH(s) {
      return `${(s / 3600).toFixed(2)} h`;
    },
    indexFromEvent(ev) {
      const rect = this.$refs.canvas.getBoundingClientRect();
      const padL = 48;
      const padR = 16;
      const x = ev.clientX - rect.left;
      const iw = rect.width - padL - padR;
      const u = Math.min(1, Math.max(0, (x - padL) / iw));
      const n = this.timeS.length;
      if (n === 0) return 0;
      return Math.round(u * (n - 1));
    },
    onPointer(ev) {
      this.$emit("update:cursor", this.indexFromEvent(ev));
      ev.target.setPointerCapture?.(ev.pointerId);
    },
    onDrag(ev) {
      if (ev.buttons !== 1) return;
      this.$emit("update:cursor", this.indexFromEvent(ev));
    },
  },
};
</script>

<style scoped>
.chart-wrap {
  position: relative;
  height: 280px;
  background: var(--paper);
  border: 1px solid var(--rule);
}
canvas {
  width: 100%;
  height: 100%;
  display: block;
  cursor: crosshair;
  touch-action: none;
}
.empty {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  color: var(--ink-soft);
  font-size: 14px;
  pointer-events: none;
}
</style>
