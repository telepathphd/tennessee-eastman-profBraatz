<template>
  <div class="trend" tabindex="0" :style="trendStyle" @keydown="onKey">
    <div class="head">
      <div class="head-left">
        <slot name="picker" />
      </div>
      <div class="tools">
        <div class="modes" role="radiogroup" aria-label="趋势排布">
          <button
            v-for="m in layouts"
            :key="m.id"
            type="button"
            role="radio"
            :aria-checked="layout === m.id"
            :class="{ on: layout === m.id }"
            @click="layout = m.id"
          >
            {{ m.label }}
          </button>
        </div>
        <button type="button" class="reset" :disabled="isFull" @click="resetView">全时段</button>
      </div>
    </div>
    <div class="body">
      <aside class="legend">
        <table>
          <thead>
            <tr>
              <th class="pin sel"></th>
              <th class="pin name">位号</th>
              <th class="desc">描述</th>
              <th class="pv">现值</th>
              <th class="lim">上限</th>
              <th class="lim">下限</th>
              <th class="auto">自动</th>
              <th class="unit">单位</th>
              <th class="drop"></th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="layer in layers"
              :key="layer.key"
              :class="{ off: layer.hidden, focus: layer.key === activeKey }"
              :style="{ '--pen': layer.color }"
              @click="onRowClick(layer)"
            >
              <td class="pin sel">
                <span class="mark">
                  <input
                    type="checkbox"
                    :checked="!layer.hidden"
                    :aria-label="`显示 ${layer.label}`"
                    @click.stop
                    @change="$emit('toggle-hidden', layer.key)"
                  />
                  <i :style="{ background: layer.color }" />
                </span>
              </td>
              <td class="pin name" :title="layer.desc || layer.label">{{ layer.label }}</td>
              <td class="desc" :title="layer.desc">{{ layer.desc || "—" }}</td>
              <td class="pv">{{ fmt(readValue(layer)) }}</td>
              <td class="lim" @click.stop>
                <input
                  type="number"
                  step="any"
                  :aria-label="`${layer.label} 上限`"
                  :placeholder="fmt(windowRange(layer)[1])"
                  :value="layer.yHi != null ? layer.yHi : ''"
                  @change="onLimitChange(layer, 'hi', $event)"
                />
              </td>
              <td class="lim" @click.stop>
                <input
                  type="number"
                  step="any"
                  :aria-label="`${layer.label} 下限`"
                  :placeholder="fmt(windowRange(layer)[0])"
                  :value="layer.yLo != null ? layer.yLo : ''"
                  @change="onLimitChange(layer, 'lo', $event)"
                />
              </td>
              <td class="auto" @click.stop>
                <button
                  type="button"
                  :class="{ on: isAuto(layer) }"
                  :disabled="isAuto(layer)"
                  title="自动缩放"
                  @click="$emit('clear-y-limits', layer.key)"
                >
                  自动
                </button>
              </td>
              <td class="unit">{{ layer.unit || "—" }}</td>
              <td class="drop">
                <button type="button" class="x" title="从列表移除" @click.stop="$emit('remove', layer.key)">
                  ×
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </aside>
      <Sash
        class="legend-sash"
        axis="v"
        @start="onLegendStart"
        @drag="onLegendDrag"
        @end="onLegendEnd"
        @reset="resetLegend"
      />
      <div class="chart">
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
          <div v-else-if="!visibleLayers.length" class="empty">勾选图层以显示纸条。</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { formatPv } from "../limits.js";
import Sash from "./Sash.vue";

const LEGEND_DEF = 336;
const LEGEND_MIN = 160;
const LEGEND_MAX = 560;
const LEGEND_STORE = "te-trend-legend-w";

export default {
  components: { Sash },
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
      layout: "overlay",
      layouts: [
        { id: "independent", label: "独立" },
        { id: "overlay", label: "叠加" },
        { id: "shared", label: "共轴" },
      ],
      viewT0: null,
      viewT1: null,
      hoverIdx: null,
      drag: null,
      markers: [],
      nearCursor: false,
      legendW: LEGEND_DEF,
      sashLegend: null,
    };
  },
  computed: {
    trendStyle() {
      return { "--legend-w": `${this.legendW}px` };
    },
    combined() {
      return this.layout !== "independent";
    },
    visibleLayers() {
      return this.layers.filter((l) => !l.hidden && l.values && l.values.length);
    },
    activeKey() {
      if (this.focusKey) {
        const layer = this.layers.find((l) => l.key === this.focusKey);
        if (layer && !layer.hidden) return this.focusKey;
      }
      const vis = this.visibleLayers[0] || this.layers[0];
      return vis?.key || "";
    },
    axisLayer() {
      const vis = this.visibleLayers;
      if (!vis.length) return null;
      return vis.find((l) => l.key === this.activeKey) || vis[0];
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
    layout: "draw",
  },
  mounted() {
    try {
      const raw = Number(localStorage.getItem(LEGEND_STORE));
      if (raw >= LEGEND_MIN && raw <= LEGEND_MAX) this.legendW = raw;
    } catch {
      /* ignore */
    }
    this.ro = new ResizeObserver(() => this.draw());
    this.ro.observe(this.$el);
    this.draw();
  },
  beforeUnmount() {
    this.ro?.disconnect();
  },
  methods: {
    fmt: formatPv,
    onLegendStart() {
      this.sashLegend = this.legendW;
    },
    onLegendDrag({ dx }) {
      if (this.sashLegend == null) return;
      this.legendW = Math.min(LEGEND_MAX, Math.max(LEGEND_MIN, this.sashLegend + dx));
    },
    onLegendEnd() {
      this.sashLegend = null;
      try {
        localStorage.setItem(LEGEND_STORE, String(this.legendW));
      } catch {
        /* ignore */
      }
    },
    resetLegend() {
      this.legendW = LEGEND_DEF;
      try {
        localStorage.setItem(LEGEND_STORE, String(LEGEND_DEF));
      } catch {
        /* ignore */
      }
    },
    onRowClick(layer) {
      if (!layer.hidden) this.$emit("focus", layer.key);
    },
    isAuto(layer) {
      return layer.yLo == null && layer.yHi == null;
    },
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
    stackGeom() {
      const canvas = this.$refs.stack;
      if (!canvas) return null;
      const w = canvas.clientWidth;
      const h = canvas.clientHeight || 200;
      const nPanes = this.combined ? 1 : Math.max(1, this.visibleLayers.length);
      const pad = { l: 52, r: 12, t: 6, b: 22 };
      const gap = this.combined ? 0 : 5;
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
    sharedYRange(layers, i0, i1) {
      let lo = Infinity;
      let hi = -Infinity;
      for (const layer of layers) {
        const [a, b] = this.yRange(layer, i0, i1);
        if (a < lo) lo = a;
        if (b > hi) hi = b;
      }
      if (!Number.isFinite(lo)) return [0, 1];
      if (hi === lo) {
        lo -= 1;
        hi += 1;
      }
      return [lo, hi];
    },
    yOfFn(y0, paneH, lo, hi) {
      const span = Math.max(1e-9, hi - lo);
      return (v) => y0 + (1 - (v - lo) / span) * paneH;
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
      this.drawStack();
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

      if (this.combined) this.drawCombined(ctx, g, layers, i0, i1, t0, t1);
      else this.drawIndependent(ctx, g, layers, i0, i1, t0, t1);

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
      let cursorX = null;
      if (ct >= t0 && ct <= t1) {
        cursorX = this.xOfTime(ct, g.pad.l, g.iw, t0, t1);
        const draggingCursor = this.drag?.mode === "cursor";
        ctx.strokeStyle = this.hoverIdx != null && !draggingCursor ? "rgba(44,51,56,0.55)" : "#2c3338";
        ctx.lineWidth = this.hoverIdx != null && !draggingCursor ? 1 : 1.6;
        ctx.beginPath();
        ctx.moveTo(cursorX, g.pad.t);
        ctx.lineTo(cursorX, g.h - 14);
        ctx.stroke();
      }

      this.drawTimeAxis(ctx, g, t0, t1, cursorX != null ? { x: cursorX, t: ct } : null);

      if (this.combined) this.drawPaneTitle(ctx, g, g.pad.t, this.axisLayer);
      else {
        layers.forEach((layer, pi) => {
          this.drawPaneTitle(ctx, g, g.pad.t + pi * (g.paneH + g.gap), layer);
        });
      }
    },
    drawTimeAxis(ctx, g, t0, t1, cursor) {
      ctx.font = "11px ui-monospace, Consolas, monospace";
      ctx.textBaseline = "alphabetic";
      const y = g.h - 6;
      const left = g.pad.l;
      const right = g.pad.l + g.iw;
      const t0s = this.fmtH(t0);
      const t1s = this.fmtH(t1);
      const t0w = ctx.measureText(t0s).width;
      const t1w = ctx.measureText(t1s).width;
      let hideT0 = false;
      let hideT1 = false;
      let cursorDraw = null;
      if (cursor) {
        const text = this.fmtCursorH(cursor.t);
        const tw = ctx.measureText(text).width;
        const padX = 5;
        const x = Math.min(right - tw / 2, Math.max(left + tw / 2, cursor.x));
        if (x - tw / 2 - padX < left + t0w + 8) hideT0 = true;
        if (x + tw / 2 + padX > right - t1w - 8) hideT1 = true;
        cursorDraw = { x, text, tw, padX };
      }
      ctx.fillStyle = "#5c656c";
      if (!hideT0) {
        ctx.textAlign = "left";
        ctx.fillText(t0s, left, y);
      }
      if (!hideT1) {
        ctx.textAlign = "right";
        ctx.fillText(t1s, right, y);
      }
      if (!cursorDraw) return;
      const { x, text, tw, padX } = cursorDraw;
      ctx.fillStyle = "#dfe3e6";
      ctx.fillRect(x - tw / 2 - padX, g.h - 18, tw + padX * 2, 16);
      ctx.fillStyle = "#2c3338";
      ctx.textAlign = "center";
      ctx.fillText(text, x, y);
    },
    drawIndependent(ctx, g, layers, i0, i1, t0, t1) {
      layers.forEach((layer, pi) => {
        const y0 = g.pad.t + pi * (g.paneH + g.gap);
        const [lo, hi] = this.yRange(layer, i0, i1);
        const yOf = this.yOfFn(y0, g.paneH, lo, hi);
        this.fillPane(ctx, g, y0, g.paneH, layer.key === this.activeKey);
        this.clipPane(ctx, g, y0, g.paneH, () => {
          this.drawBands(ctx, g, layer, yOf);
          this.drawSeries(ctx, g, layer, yOf, i0, i1, t0, t1, layer.key === this.activeKey);
        });
        this.drawYEnds(ctx, g, y0, g.paneH, lo, hi);
      });
    },
    drawCombined(ctx, g, layers, i0, i1, t0, t1) {
      const y0 = g.pad.t;
      const paneH = g.paneH;
      const axis = this.axisLayer;
      const shared = this.layout === "shared";
      const axisRange = shared ? this.sharedYRange(layers, i0, i1) : this.yRange(axis, i0, i1);
      const [alo, ahi] = axisRange;
      this.fillPane(ctx, g, y0, paneH, true);
      this.clipPane(ctx, g, y0, paneH, () => {
        this.drawYGrid(ctx, g, y0, paneH, alo, ahi);
        const bandYOf = shared ? this.yOfFn(y0, paneH, alo, ahi) : this.yOfFn(y0, paneH, ...this.yRange(axis, i0, i1));
        this.drawBands(ctx, g, axis, bandYOf);
        const rest = layers.filter((l) => l.key !== axis.key);
        for (const layer of rest) {
          const [lo, hi] = shared ? axisRange : this.yRange(layer, i0, i1);
          const yOf = this.yOfFn(y0, paneH, lo, hi);
          this.drawSeries(ctx, g, layer, yOf, i0, i1, t0, t1, false);
        }
        const [lo, hi] = shared ? axisRange : this.yRange(axis, i0, i1);
        this.drawSeries(ctx, g, axis, this.yOfFn(y0, paneH, lo, hi), i0, i1, t0, t1, true);
      });
      this.drawYAxis(ctx, g, y0, paneH, alo, ahi, shared ? "#5c656c" : axis.color);
    },
    drawPaneTitle(ctx, g, y0, layer) {
      if (!layer) return;
      const name = layer.label || "";
      ctx.font = "600 12px Bahnschrift, Segoe UI, sans-serif";
      ctx.textAlign = "left";
      ctx.textBaseline = "top";
      const x = g.pad.l + 5;
      const y = y0 + 4;
      const sw = 8;
      const gap = 5;
      const padX = 5;
      const h = 16;
      const tw = ctx.measureText(name).width;
      const w = padX + sw + gap + tw + padX;
      ctx.fillStyle = "rgba(232, 236, 239, 0.92)";
      ctx.fillRect(x, y, w, h);
      ctx.fillStyle = layer.color;
      ctx.fillRect(x + padX, y + 4, sw, 8);
      ctx.fillStyle = "#2c3338";
      ctx.fillText(name, x + padX + sw + gap, y + 1);
    },
    fillPane(ctx, g, y0, paneH, focused) {
      ctx.fillStyle = focused ? "#e8ecef" : "#d4d8dc";
      ctx.fillRect(g.pad.l, y0, g.iw, paneH);
      ctx.strokeStyle = focused ? "#3d6b99" : "#9aa1a6";
      ctx.lineWidth = focused ? 1.4 : 1;
      ctx.strokeRect(g.pad.l, y0, g.iw, paneH);
    },
    clipPane(ctx, g, y0, paneH, fn) {
      ctx.save();
      ctx.beginPath();
      ctx.rect(g.pad.l, y0, g.iw, paneH);
      ctx.clip();
      fn();
      ctx.restore();
    },
    drawBands(ctx, g, layer, yOf) {
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
    },
    drawSeries(ctx, g, layer, yOf, i0, i1, t0, t1, focused) {
      ctx.lineJoin = "round";
      ctx.lineCap = "round";
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
      this.strokePen(ctx, layer.color, focused);
    },
    strokePen(ctx, color, focused) {
      const overlay = focused && this.combined;
      const width = overlay ? 2.5 : focused ? 2.2 : 1.5;
      if (overlay) {
        ctx.strokeStyle = "rgba(255, 255, 255, 0.95)";
        ctx.lineWidth = width + 3.2;
        ctx.stroke();
        ctx.shadowColor = "rgba(255, 255, 255, 0.9)";
        ctx.shadowBlur = 5;
      }
      ctx.strokeStyle = color;
      ctx.lineWidth = width;
      ctx.stroke();
      if (overlay) {
        ctx.shadowColor = "transparent";
        ctx.shadowBlur = 0;
      }
    },
    drawYEnds(ctx, g, y0, paneH, lo, hi) {
      ctx.fillStyle = "#5c656c";
      ctx.font = "10px ui-monospace, Consolas, monospace";
      ctx.textAlign = "right";
      ctx.textBaseline = "top";
      ctx.fillText(this.fmt(hi), g.pad.l - 4, y0 + 1);
      ctx.textBaseline = "bottom";
      ctx.fillText(this.fmt(lo), g.pad.l - 4, y0 + paneH - 1);
    },
    drawYGrid(ctx, g, y0, paneH, lo, hi) {
      const yOf = this.yOfFn(y0, paneH, lo, hi);
      const nTicks = paneH > 90 ? 4 : 2;
      ctx.strokeStyle = "rgba(44,51,56,0.1)";
      ctx.lineWidth = 1;
      for (let i = 1; i < nTicks; i++) {
        const y = yOf(hi - (i / nTicks) * (hi - lo));
        ctx.beginPath();
        ctx.moveTo(g.pad.l, y);
        ctx.lineTo(g.pad.l + g.iw, y);
        ctx.stroke();
      }
    },
    drawYAxis(ctx, g, y0, paneH, lo, hi, color) {
      const yOf = this.yOfFn(y0, paneH, lo, hi);
      const nTicks = paneH > 90 ? 4 : 2;
      ctx.strokeStyle = color;
      ctx.lineWidth = 1.4;
      ctx.beginPath();
      ctx.moveTo(g.pad.l, y0);
      ctx.lineTo(g.pad.l, y0 + paneH);
      ctx.stroke();
      ctx.fillStyle = color;
      ctx.font = "10px ui-monospace, Consolas, monospace";
      ctx.textAlign = "right";
      for (let i = 0; i <= nTicks; i++) {
        const v = hi - (i / nTicks) * (hi - lo);
        const y = yOf(v);
        ctx.textBaseline = i === 0 ? "top" : i === nTicks ? "bottom" : "middle";
        ctx.fillText(this.fmt(v), g.pad.l - 4, y);
      }
    },
    fmtH(s) {
      const h = s / 3600;
      return this.t1 - this.t0 < 1800 ? `${h.toFixed(3)} h` : `${h.toFixed(2)} h`;
    },
    fmtCursorH(s) {
      const h = s / 3600;
      return this.t1 - this.t0 < 600 ? `${h.toFixed(4)} h` : `${h.toFixed(3)} h`;
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
.head {
  display: grid;
  grid-template-columns: var(--legend-w) 5px minmax(0, 1fr);
  align-items: center;
  gap: 0;
  padding: 0.25rem 0.5rem 0;
}
.head-left {
  grid-column: 1;
  min-width: 0;
}
.tools {
  grid-column: 3;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.4rem;
  flex: 0 0 auto;
}
.modes {
  display: flex;
  border: 1px solid var(--ink);
}
.modes button {
  border: 0;
  border-right: 1px solid var(--ink);
  background: transparent;
  color: var(--ink);
  padding: 0.15rem 0.45rem;
  cursor: pointer;
  font-size: 0.72rem;
}
.modes button:last-child {
  border-right: 0;
}
.modes button.on {
  background: var(--ink);
  color: var(--panel);
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
  grid-template-columns: var(--legend-w) 5px minmax(0, 1fr);
  flex: 1;
  min-height: 0;
}
.legend-sash {
  grid-column: 2;
  grid-row: 1;
}
.chart {
  grid-column: 3;
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}
.legend {
  grid-column: 1;
  overflow: auto;
  min-width: 0;
  min-height: 0;
  background: var(--panel);
}
.legend table {
  border-collapse: separate;
  border-spacing: 0;
  width: max-content;
  min-width: 100%;
}
.legend th,
.legend td {
  padding: 0.18rem 0.28rem;
  font-size: 0.72rem;
  white-space: nowrap;
  vertical-align: middle;
  border-bottom: 1px solid var(--rule);
  background: var(--panel);
}
.legend th {
  position: sticky;
  top: 0;
  z-index: 2;
  font-weight: 600;
  font-size: 0.62rem;
  letter-spacing: 0.06em;
  color: var(--ink-soft);
  text-align: left;
  background: #d8dce0;
}
.legend .pin {
  position: sticky;
  z-index: 1;
  left: 0;
}
.legend th.pin {
  z-index: 3;
}
.legend .pin.sel {
  left: 0;
  width: 1.55rem;
  min-width: 1.55rem;
  padding-right: 0.1rem;
}
.legend .pin.name {
  left: 1.55rem;
  min-width: 5.4rem;
  max-width: 5.4rem;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: var(--type-data);
  letter-spacing: 0.02em;
  box-shadow: 4px 0 6px -4px rgba(44, 51, 56, 0.28);
}
.legend .desc {
  max-width: 7.2rem;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--ink-soft);
}
.legend tbody tr {
  cursor: pointer;
}
.legend tbody tr:hover:not(.focus) td {
  background: #eceef0;
}
.legend tbody tr.focus td {
  background: #c5d0dc;
}
.legend tbody tr.focus td.pin.sel {
  box-shadow: inset 4px 0 0 var(--pen, var(--event));
}
.legend tbody tr.focus .pin.name {
  font-weight: 700;
  color: var(--ink);
}
.legend tbody tr.focus .mark i {
  outline: 1px solid var(--ink);
  outline-offset: 1px;
}
.legend tbody tr.off {
  opacity: 0.45;
}
.legend .mark {
  display: flex;
  align-items: center;
  gap: 0.22rem;
}
.legend .mark input {
  width: 0.75rem;
  height: 0.75rem;
  margin: 0;
  accent-color: var(--ink);
}
.legend .mark i {
  width: 0.62rem;
  height: 0.62rem;
  flex: 0 0 auto;
  display: inline-block;
}
.legend .pv {
  font-family: var(--type-data);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  text-align: right;
  min-width: 3.6rem;
}
.legend th.pv,
.legend th.lim,
.legend th.unit {
  text-align: right;
}
.legend th.auto {
  text-align: center;
}
.legend .lim {
  width: 4.4rem;
}
.legend .lim input {
  width: 4.2rem;
  min-width: 0;
  border: 1px solid var(--rule);
  background: #f3f5f6;
  padding: 0.08rem 0.22rem;
  color: var(--ink);
  font-family: var(--type-data);
  font-size: 0.72rem;
  text-align: right;
  appearance: textfield;
}
.legend .lim input::-webkit-outer-spin-button,
.legend .lim input::-webkit-inner-spin-button {
  appearance: none;
  margin: 0;
}
.legend .auto {
  text-align: center;
}
.legend .auto button {
  border: 1px solid var(--ink);
  background: transparent;
  color: var(--ink);
  padding: 0.06rem 0.28rem;
  cursor: pointer;
  font-size: 0.68rem;
}
.legend .auto button.on,
.legend .auto button:disabled {
  background: var(--ink);
  color: var(--panel);
  opacity: 1;
  cursor: default;
}
.legend .unit {
  color: var(--ink-soft);
  min-width: 3.2rem;
}
.legend .drop {
  width: 1.2rem;
  padding-left: 0.08rem;
  padding-right: 0.2rem;
}
.legend .x {
  border: 0;
  background: transparent;
  color: var(--ink-soft);
  cursor: pointer;
  padding: 0 0.1rem;
  font-size: 0.9rem;
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
