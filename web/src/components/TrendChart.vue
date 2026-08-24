<template>
  <div class="trend" tabindex="0" @keydown="onKey">
    <div class="toolbar">
      <span class="hint">滚轮缩放 · 拖移时间窗 · 拖时刻光标 · 点注入/联锁跳转</span>
      <button type="button" class="reset" :disabled="isFull" @click="resetView">全时段</button>
    </div>
    <div class="body">
      <div class="chart">
        <canvas
          ref="overview"
          class="overview"
          @pointerdown="onOverviewDown"
          @pointermove="onOverviewMove"
          @pointerup="onOverviewUp"
        />
        <div class="stack-wrap">
          <canvas
            ref="stack"
            class="stack"
            :style="{ cursor: stackCursor }"
            @pointerdown="onStackDown"
            @pointermove="onStackMove"
            @pointerup="onStackUp"
            @pointercancel="onStackUp"
            @pointerleave="onLeave"
            @wheel.prevent="onWheel"
            @dblclick.prevent="resetView"
          />
          <div v-if="!layers.length" class="empty">从流程图或关键变量拖入测量</div>
          <div v-else-if="timeS.length < 2" class="empty">运行仿真后，纸条上才会出现过程值。</div>
          <div v-else-if="!visibleLayers.length" class="empty">勾选右侧图层以显示纸条。</div>
        </div>
      </div>
      <aside class="legend">
        <div
          v-for="layer in layers"
          :key="layer.key"
          class="layer"
          :class="{ off: layer.hidden, focus: layer.key === focusKey }"
          @click="$emit('focus', layer.key)"
        >
          <div class="head">
            <input
              type="checkbox"
              :checked="!layer.hidden"
              @click.stop
              @change="$emit('toggle-hidden', layer.key)"
            />
            <i :style="{ background: layer.color }" />
            <span class="name">{{ layer.label }}</span>
            <button type="button" class="x" title="从列表移除" @click.stop="$emit('remove', layer.key)">
              ×
            </button>
          </div>
          <div class="read">
            <b>{{ fmt(readValue(layer)) }}</b>
            <small>{{ layer.unit }}</small>
          </div>
          <div class="lims" @click.stop>
            <label>
              下限
              <input
                type="number"
                step="any"
                :placeholder="fmt(windowRange(layer)[0])"
                :value="layer.yLo != null ? layer.yLo : ''"
                @change="onLimitChange(layer, 'lo', $event)"
              />
            </label>
            <label>
              上限
              <input
                type="number"
                step="any"
                :placeholder="fmt(windowRange(layer)[1])"
                :value="layer.yHi != null ? layer.yHi : ''"
                @change="onLimitChange(layer, 'hi', $event)"
              />
            </label>
            <button
              v-if="layer.yLo != null || layer.yHi != null"
              type="button"
              class="auto"
              @click="$emit('clear-y-limits', layer.key)"
            >
              自动
            </button>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<script>
import { formatPv } from "../limits.js";

export default {
  props: {
    timeS: { type: Array, default: () => [] },
    layers: { type: Array, default: () => [] },
    cursor: { type: Number, default: 0 },
    injections: { type: Array, default: () => [] },
    shutdownTimeS: { type: Number, default: null },
    focusKey: { type: String, default: "" },
  },
  emits: ["update:cursor", "toggle-hidden", "remove", "focus", "set-y-limits", "clear-y-limits"],
  data() {
    return {
      viewT0: null,
      viewT1: null,
      hoverIdx: null,
      drag: null,
      ovDrag: null,
      markers: [],
      nearCursor: false,
    };
  },
  computed: {
    visibleLayers() {
      return this.layers.filter((l) => !l.hidden && l.values && l.values.length);
    },
    tFull0() {
      return this.timeS[0] ?? 0;
    },
    tFull1() {
      const n = this.timeS.length;
      return n ? this.timeS[n - 1] : 1;
    },
    t0() {
      return this.viewT0 == null ? this.tFull0 : this.viewT0;
    },
    t1() {
      return this.viewT1 == null ? this.tFull1 : this.viewT1;
    },
    isFull() {
      if (this.viewT0 == null) return true;
      return this.t0 <= this.tFull0 + 1e-6 && this.t1 >= this.tFull1 - 1e-6;
    },
    readIdx() {
      return this.hoverIdx != null ? this.hoverIdx : this.cursor;
    },
    stackCursor() {
      if (this.drag?.mode === "pan") return "grabbing";
      if (this.drag?.mode === "cursor" || this.nearCursor) return "ew-resize";
      return "grab";
    },
  },
  watch: {
    timeS() {
      this.viewT0 = null;
      this.viewT1 = null;
      this.$nextTick(() => this.draw());
    },
    layers: { handler: "draw", deep: true },
    cursor: "draw",
    injections: "draw",
    shutdownTimeS: "draw",
    focusKey: "draw",
    hoverIdx: "draw",
    viewT0: "draw",
    viewT1: "draw",
  },
  mounted() {
    this.ro = new ResizeObserver(() => this.draw());
    this.ro.observe(this.$el);
    this.draw();
  },
  beforeUnmount() {
    this.ro?.disconnect();
  },
  methods: {
    fmt: formatPv,
    readValue(layer) {
      return layer.values?.[this.readIdx];
    },
    minSpan() {
      const full = Math.max(1, this.tFull1 - this.tFull0);
      const dt = this.timeS.length > 1 ? this.timeS[1] - this.timeS[0] : 1;
      return Math.max(dt * 2, Math.min(30, full));
    },
    clampView(a, b) {
      let lo = Math.min(a, b);
      let hi = Math.max(a, b);
      const span = Math.max(this.minSpan(), hi - lo);
      hi = lo + span;
      if (hi > this.tFull1) {
        hi = this.tFull1;
        lo = hi - span;
      }
      if (lo < this.tFull0) {
        lo = this.tFull0;
        hi = Math.min(this.tFull1, lo + span);
      }
      this.viewT0 = lo;
      this.viewT1 = hi;
    },
    resetView() {
      this.viewT0 = null;
      this.viewT1 = null;
    },
    zoomAround(tFocus, factor) {
      const span = this.t1 - this.t0;
      const next = Math.min(this.tFull1 - this.tFull0, Math.max(this.minSpan(), span * factor));
      const u = span > 0 ? (tFocus - this.t0) / span : 0.5;
      const lo = tFocus - u * next;
      this.clampView(lo, lo + next);
    },
    jumpTo(t) {
      const full = this.tFull1 - this.tFull0;
      const span = Math.max(this.minSpan(), Math.min(full, Math.max(full * 0.12, 1800)));
      this.clampView(t - span / 2, t + span / 2);
      this.$emit("update:cursor", this.indexAtTime(t));
    },
    indexAtTime(t) {
      const n = this.timeS.length;
      if (!n) return 0;
      let lo = 0;
      let hi = n - 1;
      if (t <= this.timeS[0]) return 0;
      if (t >= this.timeS[hi]) return hi;
      while (hi - lo > 1) {
        const mid = (lo + hi) >> 1;
        if (this.timeS[mid] <= t) lo = mid;
        else hi = mid;
      }
      return t - this.timeS[lo] <= this.timeS[hi] - t ? lo : hi;
    },
    timeFromX(x, padL, iw) {
      const u = Math.min(1, Math.max(0, (x - padL) / Math.max(1, iw)));
      return this.t0 + u * (this.t1 - this.t0);
    },
    indexFromX(x, padL, iw) {
      return this.indexAtTime(this.timeFromX(x, padL, iw));
    },
    xOfTime(t, padL, iw, t0, t1) {
      return padL + ((t - t0) / Math.max(1e-9, t1 - t0)) * iw;
    },
    overviewGeom() {
      const canvas = this.$refs.overview;
      if (!canvas) return null;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight || 36;
      const pad = { l: 52, r: 12, t: 4, b: 4 };
      return { canvas, w, h, pad, iw: w - pad.l - pad.r, ih: h - pad.t - pad.b };
    },
    stackGeom() {
      const canvas = this.$refs.stack;
      if (!canvas) return null;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight || 200;
      const nPanes = Math.max(1, this.visibleLayers.length);
      const pad = { l: 52, r: 12, t: 6, b: 22 };
      const gap = 5;
      const paneH = Math.max(28, (h - pad.t - pad.b - gap * (nPanes - 1)) / nPanes);
      return { canvas, w, h, pad, gap, paneH, iw: w - pad.l - pad.r };
    },
    sizeCanvas(canvas, w, h) {
      const dpr = window.devicePixelRatio || 1;
      canvas.width = Math.max(1, Math.floor(w * dpr));
      canvas.height = Math.max(1, Math.floor(h * dpr));
      const ctx = canvas.getContext("2d");
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      return ctx;
    },
    dataRange(layer, i0, i1) {
      let lo = Infinity;
      let hi = -Infinity;
      const values = layer.values || [];
      const a = Math.max(0, i0);
      const b = Math.min(values.length - 1, i1);
      for (let i = a; i <= b; i++) {
        const v = values[i];
        if (v == null || Number.isNaN(v)) continue;
        if (v < lo) lo = v;
        if (v > hi) hi = v;
      }
      if (!Number.isFinite(lo)) {
        lo = 0;
        hi = 1;
      }
      if (hi === lo) {
        lo -= 1;
        hi += 1;
      }
      const pad = (hi - lo) * 0.08;
      return [lo - pad, hi + pad];
    },
    windowRange(layer) {
      const n = this.timeS.length;
      if (n < 2 || !layer.values?.length) return [0, 1];
      return this.dataRange(layer, this.indexAtTime(this.t0), this.indexAtTime(this.t1));
    },
    yRange(layer, i0, i1) {
      if (layer.yLo != null && layer.yHi != null && Number.isFinite(layer.yLo) && Number.isFinite(layer.yHi)) {
        let lo = layer.yLo;
        let hi = layer.yHi;
        if (lo === hi) {
          lo -= 1;
          hi += 1;
        }
        return lo < hi ? [lo, hi] : [hi, lo];
      }
      return this.dataRange(layer, i0, i1);
    },
    onLimitChange(layer, side, ev) {
      const n = Number(ev.target.value);
      if (!Number.isFinite(n)) {
        ev.target.value = layer[side === "lo" ? "yLo" : "yHi"] ?? "";
        return;
      }
      const auto = this.windowRange(layer);
      let lo = side === "lo" ? n : (layer.yLo ?? auto[0]);
      let hi = side === "hi" ? n : (layer.yHi ?? auto[1]);
      if (lo === hi) hi = lo + 1;
      if (lo > hi) [lo, hi] = [hi, lo];
      this.$emit("set-y-limits", { key: layer.key, lo, hi });
    },
    draw() {
      this.drawOverview();
      this.drawStack();
    },
    drawOverview() {
      const g = this.overviewGeom();
      if (!g) return;
      const ctx = this.sizeCanvas(g.canvas, g.w, g.h);
      ctx.fillStyle = "#cfd3d6";
      ctx.fillRect(0, 0, g.w, g.h);
      const n = this.timeS.length;
      if (n < 2 || !this.visibleLayers.length) return;
      const t0 = this.tFull0;
      const t1 = this.tFull1;
      this.visibleLayers.slice(0, 4).forEach((layer) => {
        const [lo, hi] = this.yRange(layer, 0, n - 1);
        ctx.strokeStyle = layer.color;
        ctx.lineWidth = 1;
        ctx.beginPath();
        const step = Math.max(1, Math.floor(n / Math.max(g.iw, 1)));
        for (let i = 0; i < n; i += step) {
          const x = this.xOfTime(this.timeS[i], g.pad.l, g.iw, t0, t1);
          const y = g.pad.t + (1 - (layer.values[i] - lo) / (hi - lo)) * g.ih;
          if (i === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        }
        ctx.stroke();
      });
      const x0 = this.xOfTime(this.t0, g.pad.l, g.iw, t0, t1);
      const x1 = this.xOfTime(this.t1, g.pad.l, g.iw, t0, t1);
      ctx.fillStyle = "rgba(44, 51, 56, 0.18)";
      ctx.fillRect(x0, g.pad.t, Math.max(2, x1 - x0), g.ih);
      ctx.strokeStyle = "#2c3338";
      ctx.strokeRect(x0, g.pad.t, Math.max(2, x1 - x0), g.ih);
    },
    drawStack() {
      const g = this.stackGeom();
      if (!g) return;
      const ctx = this.sizeCanvas(g.canvas, g.w, g.h);
      ctx.fillStyle = "#dfe3e6";
      ctx.fillRect(0, 0, g.w, g.h);
      const n = this.timeS.length;
      const layers = this.visibleLayers;
      this.markers = [];
      if (n < 2 || !layers.length) return;
      const i0 = this.indexAtTime(this.t0);
      const i1 = this.indexAtTime(this.t1);
      const t0 = this.t0;
      const t1 = this.t1;

      layers.forEach((layer, pi) => {
        const y0 = g.pad.t + pi * (g.paneH + g.gap);
        const focused = layer.key === this.focusKey;
        ctx.fillStyle = focused ? "#e8ecef" : "#d4d8dc";
        ctx.fillRect(g.pad.l, y0, g.iw, g.paneH);
        ctx.strokeStyle = focused ? "#3d6b99" : "#9aa1a6";
        ctx.lineWidth = focused ? 1.4 : 1;
        ctx.strokeRect(g.pad.l, y0, g.iw, g.paneH);

        const [lo, hi] = this.yRange(layer, i0, i1);
        const span = Math.max(1e-9, hi - lo);
        const yOf = (v) => y0 + (1 - (v - lo) / span) * g.paneH;

        ctx.save();
        ctx.beginPath();
        ctx.rect(g.pad.l, y0, g.iw, g.paneH);
        ctx.clip();

        if (layer.loOp != null && layer.hiOp != null) {
          ctx.fillStyle = "rgba(184, 192, 198, 0.35)";
          const ya = yOf(layer.hiOp);
          const yb = yOf(layer.loOp);
          ctx.fillRect(g.pad.l, Math.min(ya, yb), g.iw, Math.abs(yb - ya));
        }
        ctx.setLineDash([3, 3]);
        ctx.lineWidth = 1;
        ctx.strokeStyle = "#c42b2b";
        if (layer.hiShutdown != null) {
          ctx.beginPath();
          ctx.moveTo(g.pad.l, yOf(layer.hiShutdown));
          ctx.lineTo(g.pad.l + g.iw, yOf(layer.hiShutdown));
          ctx.stroke();
        }
        if (layer.loShutdown != null) {
          ctx.beginPath();
          ctx.moveTo(g.pad.l, yOf(layer.loShutdown));
          ctx.lineTo(g.pad.l + g.iw, yOf(layer.loShutdown));
          ctx.stroke();
        }
        ctx.setLineDash([]);

        ctx.strokeStyle = layer.color;
        ctx.lineWidth = 1.5;
        ctx.beginPath();
        const step = Math.max(1, Math.floor((i1 - i0 + 1) / Math.max(g.iw * 2, 1)));
        let started = false;
        for (let i = i0; i <= i1; i += step) {
          const x = this.xOfTime(this.timeS[i], g.pad.l, g.iw, t0, t1);
          const y = yOf(layer.values[i]);
          if (!started) {
            ctx.moveTo(x, y);
            started = true;
          } else ctx.lineTo(x, y);
        }
        if ((i1 - i0) % step !== 0) {
          ctx.lineTo(this.xOfTime(this.timeS[i1], g.pad.l, g.iw, t0, t1), yOf(layer.values[i1]));
        }
        ctx.stroke();
        ctx.restore();

        ctx.fillStyle = "#5c656c";
        ctx.font = "10px ui-monospace, Consolas, monospace";
        ctx.textAlign = "right";
        ctx.textBaseline = "top";
        ctx.fillText(this.fmt(hi), g.pad.l - 4, y0 + 1);
        ctx.textBaseline = "bottom";
        ctx.fillText(this.fmt(lo), g.pad.l - 4, y0 + g.paneH - 1);
        ctx.textAlign = "left";
        ctx.textBaseline = "top";
        ctx.fillStyle = "#2c3338";
        ctx.font = "11px Bahnschrift, Segoe UI, sans-serif";
        ctx.fillText(layer.label, g.pad.l + 6, y0 + 4);
      });

      const markEvent = (t, color, dash, label) => {
        if (t == null || t < t0 || t > t1) return;
        const x = this.xOfTime(t, g.pad.l, g.iw, t0, t1);
        ctx.strokeStyle = color;
        ctx.lineWidth = 1.2;
        ctx.setLineDash(dash);
        ctx.beginPath();
        ctx.moveTo(x, g.pad.t);
        ctx.lineTo(x, g.h - g.pad.b);
        ctx.stroke();
        ctx.setLineDash([]);
        ctx.fillStyle = color;
        ctx.font = "10px Bahnschrift, Segoe UI, sans-serif";
        ctx.textAlign = "left";
        ctx.textBaseline = "top";
        ctx.fillText(label, x + 3, g.pad.t);
        this.markers.push({ x, t });
      };
      for (const inj of this.injections) {
        markEvent(inj.start_step, "#3d6b99", [4, 3], `IDV${inj.idv}`);
      }
      if (this.shutdownTimeS != null) {
        markEvent(this.shutdownTimeS, "#c42b2b", [], "联锁");
      }

      const idx = this.hoverIdx != null ? this.hoverIdx : this.cursor;
      const ct = this.timeS[Math.min(idx, n - 1)];
      if (ct >= t0 && ct <= t1) {
        const cx = this.xOfTime(ct, g.pad.l, g.iw, t0, t1);
        const draggingCursor = this.drag?.mode === "cursor";
        ctx.strokeStyle = this.hoverIdx != null && !draggingCursor ? "rgba(44,51,56,0.55)" : "#2c3338";
        ctx.lineWidth = this.hoverIdx != null && !draggingCursor ? 1 : 1.6;
        ctx.beginPath();
        ctx.moveTo(cx, g.pad.t);
        ctx.lineTo(cx, g.h - g.pad.b);
        ctx.stroke();
      }

      ctx.fillStyle = "#5c656c";
      ctx.font = "11px ui-monospace, Consolas, monospace";
      ctx.textBaseline = "alphabetic";
      ctx.textAlign = "left";
      ctx.fillText(this.fmtH(t0), g.pad.l, g.h - 6);
      ctx.textAlign = "right";
      ctx.fillText(this.fmtH(t1), g.pad.l + g.iw, g.h - 6);
    },
    fmtH(s) {
      const h = s / 3600;
      return this.t1 - this.t0 < 1800 ? `${h.toFixed(3)} h` : `${h.toFixed(2)} h`;
    },
    localXY(ev, canvas) {
      const r = canvas.getBoundingClientRect();
      return { x: ev.clientX - r.left, y: ev.clientY - r.top };
    },
    hitMarker(x) {
      return this.markers.find((m) => Math.abs(m.x - x) <= 8) || null;
    },
    cursorX(g) {
      const n = this.timeS.length;
      if (!n) return null;
      const ct = this.timeS[Math.min(this.cursor, n - 1)];
      if (ct < this.t0 || ct > this.t1) return null;
      return this.xOfTime(ct, g.pad.l, g.iw, this.t0, this.t1);
    },
    hitCursor(x, g) {
      const cx = this.cursorX(g);
      return cx != null && Math.abs(x - cx) <= 8;
    },
    onStackDown(ev) {
      const g = this.stackGeom();
      if (!g) return;
      const { x } = this.localXY(ev, g.canvas);
      g.canvas.setPointerCapture?.(ev.pointerId);
      if (this.hitCursor(x, g)) {
        this.hoverIdx = null;
        this.drag = { x, mode: "cursor" };
        this.$emit("update:cursor", this.indexFromX(x, g.pad.l, g.iw));
        return;
      }
      this.drag = { x, mode: "maybe", t0: this.t0, t1: this.t1 };
    },
    onStackMove(ev) {
      const g = this.stackGeom();
      if (!g) return;
      const { x } = this.localXY(ev, g.canvas);
      if (!this.drag) {
        this.hoverIdx = this.indexFromX(x, g.pad.l, g.iw);
        this.nearCursor = this.hitCursor(x, g);
        return;
      }
      if (this.drag.mode === "cursor") {
        this.$emit("update:cursor", this.indexFromX(x, g.pad.l, g.iw));
        return;
      }
      if (this.drag.mode === "maybe" && Math.abs(x - this.drag.x) > 5) this.drag.mode = "pan";
      if (this.drag.mode === "pan") {
        const dt = this.timeFromX(this.drag.x, g.pad.l, g.iw) - this.timeFromX(x, g.pad.l, g.iw);
        this.clampView(this.drag.t0 + dt, this.drag.t1 + dt);
      }
    },
    onStackUp(ev) {
      const g = this.stackGeom();
      const drag = this.drag;
      this.drag = null;
      if (!g || !drag) return;
      if (drag.mode === "pan" || drag.mode === "cursor") return;
      const { x } = this.localXY(ev, g.canvas);
      const mark = this.hitMarker(x);
      if (mark) {
        this.jumpTo(mark.t);
        return;
      }
      this.$emit("update:cursor", this.indexFromX(x, g.pad.l, g.iw));
    },
    onLeave() {
      if (!this.drag) {
        this.hoverIdx = null;
        this.nearCursor = false;
      }
    },
    onWheel(ev) {
      const g = this.stackGeom();
      if (!g) return;
      const { x } = this.localXY(ev, g.canvas);
      this.zoomAround(this.timeFromX(x, g.pad.l, g.iw), ev.deltaY > 0 ? 1.18 : 0.85);
    },
    onOverviewDown(ev) {
      const g = this.overviewGeom();
      if (!g) return;
      const { x } = this.localXY(ev, g.canvas);
      g.canvas.setPointerCapture?.(ev.pointerId);
      const x0 = this.xOfTime(this.t0, g.pad.l, g.iw, this.tFull0, this.tFull1);
      const x1 = this.xOfTime(this.t1, g.pad.l, g.iw, this.tFull0, this.tFull1);
      let mode = "move";
      if (Math.abs(x - x0) < 7) mode = "l";
      else if (Math.abs(x - x1) < 7) mode = "r";
      else if (x < x0 || x > x1) {
        const t = this.tFull0 + ((x - g.pad.l) / g.iw) * (this.tFull1 - this.tFull0);
        const span = this.t1 - this.t0;
        this.clampView(t - span / 2, t + span / 2);
      }
      this.ovDrag = { mode, x, t0: this.t0, t1: this.t1 };
    },
    onOverviewMove(ev) {
      if (!this.ovDrag) return;
      const g = this.overviewGeom();
      if (!g) return;
      const { x } = this.localXY(ev, g.canvas);
      const dt = ((x - this.ovDrag.x) / g.iw) * (this.tFull1 - this.tFull0);
      if (this.ovDrag.mode === "move") this.clampView(this.ovDrag.t0 + dt, this.ovDrag.t1 + dt);
      else if (this.ovDrag.mode === "l") this.clampView(this.ovDrag.t0 + dt, this.ovDrag.t1);
      else this.clampView(this.ovDrag.t0, this.ovDrag.t1 + dt);
    },
    onOverviewUp() {
      this.ovDrag = null;
    },
    onKey(ev) {
      const n = this.timeS.length;
      if (!n) return;
      if (ev.key === "ArrowLeft") {
        ev.preventDefault();
        this.$emit("update:cursor", Math.max(0, this.cursor - 1));
      } else if (ev.key === "ArrowRight") {
        ev.preventDefault();
        this.$emit("update:cursor", Math.min(n - 1, this.cursor + 1));
      } else if (ev.key === "Home") {
        ev.preventDefault();
        this.$emit("update:cursor", 0);
      } else if (ev.key === "End") {
        ev.preventDefault();
        this.$emit("update:cursor", n - 1);
      } else if (ev.key === "Escape") {
        this.resetView();
      }
    },
  },
};
</script>

<style scoped>
.trend {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 180px;
  height: auto;
  background: var(--panel);
  outline: none;
}
.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.6rem;
  padding: 0.25rem 0.5rem 0;
  font-size: 0.72rem;
  color: var(--ink-soft);
}
.reset {
  border: 1px solid var(--ink);
  background: transparent;
  color: var(--ink);
  padding: 0.15rem 0.45rem;
  cursor: pointer;
  font-size: 0.72rem;
}
.reset:disabled {
  opacity: 0.4;
  cursor: default;
}
.body {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 200px;
  flex: 1;
  min-height: 0;
}
.chart {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}
.legend {
  overflow: auto;
  border-left: 1px solid var(--rule);
  padding: 0.25rem 0.35rem 0.4rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  min-height: 0;
}
.layer {
  display: flex;
  flex-direction: column;
  gap: 0.18rem;
  margin: 0;
  padding: 0.28rem 0.3rem;
  border: 1px solid transparent;
  cursor: pointer;
}
.layer:hover,
.layer.focus {
  border-color: var(--rule);
  background: #eceef0;
}
.layer.off {
  opacity: 0.45;
}
.head {
  display: flex;
  align-items: center;
  gap: 0.28rem;
  min-width: 0;
}
.head i {
  width: 0.65rem;
  height: 0.65rem;
  flex: 0 0 auto;
  display: inline-block;
}
.name {
  flex: 1;
  min-width: 0;
  font-size: 0.72rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.layer.focus .name {
  text-decoration: underline;
  text-underline-offset: 2px;
}
.x {
  border: 0;
  background: transparent;
  color: var(--ink-soft);
  cursor: pointer;
  padding: 0 0.15rem;
  font-size: 0.9rem;
}
.read {
  display: flex;
  align-items: baseline;
  gap: 0.3rem;
  padding-left: 1.35rem;
}
.read b {
  font-family: var(--type-data);
  font-weight: 600;
  font-size: 0.88rem;
}
.read small {
  color: var(--ink-soft);
  font-size: 0.68rem;
}
.lims {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.2rem 0.28rem;
  padding-left: 1.35rem;
}
.lims label {
  display: flex;
  flex-direction: column;
  gap: 0.08rem;
  margin: 0;
  font-size: 0.62rem;
  color: var(--ink-soft);
}
.lims input {
  width: 100%;
  min-width: 0;
  border: 1px solid var(--rule);
  background: #f3f5f6;
  padding: 0.12rem 0.25rem;
  color: var(--ink);
  font-family: var(--type-data);
  font-size: 0.72rem;
}
.auto {
  grid-column: 1 / -1;
  border: 1px solid var(--ink);
  background: transparent;
  color: var(--ink);
  padding: 0.1rem 0.3rem;
  cursor: pointer;
  font-size: 0.68rem;
}
.overview {
  height: 36px;
  width: 100%;
  display: block;
  cursor: ew-resize;
  touch-action: none;
}
.stack-wrap {
  position: relative;
  flex: 1;
  min-height: 140px;
}
.stack {
  width: 100%;
  height: 100%;
  display: block;
  touch-action: none;
}
.empty {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  color: var(--ink-soft);
  font-size: 0.9rem;
  pointer-events: none;
}
</style>
