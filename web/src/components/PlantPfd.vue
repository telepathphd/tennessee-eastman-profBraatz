<template>
  <svg class="pfd" viewBox="0 0 1120 520" role="img" aria-label="Tennessee Eastman 工艺流程图">
    <defs>
      <marker id="arr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto">
        <path d="M0 0 L10 5 L0 10 z" fill="#6a737b" />
      </marker>
    </defs>
    <rect width="1120" height="520" fill="var(--panel)" />

    <g class="stream">
      <path d="M40 110 H170" marker-end="url(#arr)" />
      <path d="M40 150 H170" marker-end="url(#arr)" />
      <path d="M40 190 H170" marker-end="url(#arr)" />
      <path d="M40 230 H170" marker-end="url(#arr)" />
    </g>

    <g class="hot" :class="hotClass(1)" @pointerdown="onHotDown($event, 1)">
      <title>{{ desc(1) }}</title>
      <rect class="hit" x="36" y="86" width="128" height="28" />
      <text x="44" y="104">{{ tag(1) }}</text>
      <text class="val" :class="valClass(1)" x="100" y="104">{{ m(1) }}</text>
    </g>
    <g class="hot" :class="hotClass(2)" @pointerdown="onHotDown($event, 2)">
      <title>{{ desc(2) }}</title>
      <rect class="hit" x="36" y="126" width="128" height="28" />
      <text x="44" y="144">{{ tag(2) }}</text>
      <text class="val" :class="valClass(2)" x="100" y="144">{{ m(2) }}</text>
    </g>
    <g class="hot" :class="hotClass(3)" @pointerdown="onHotDown($event, 3)">
      <title>{{ desc(3) }}</title>
      <rect class="hit" x="36" y="166" width="128" height="28" />
      <text x="44" y="184">{{ tag(3) }}</text>
      <text class="val" :class="valClass(3)" x="100" y="184">{{ m(3) }}</text>
    </g>
    <g class="hot" :class="hotClass(4)" @pointerdown="onHotDown($event, 4)">
      <title>{{ desc(4) }}</title>
      <rect class="hit" x="36" y="206" width="128" height="28" />
      <text x="44" y="224">{{ tag(4) }}</text>
      <text class="val" :class="valClass(4)" x="100" y="224">{{ m(4) }}</text>
    </g>

    <g class="equip" transform="translate(170,70)">
      <rect x="0" y="20" width="170" height="200" rx="86" />
      <line x1="20" y1="70" x2="150" y2="70" />
      <circle cx="85" cy="130" r="14" />
      <line x1="85" y1="116" x2="85" y2="144" />
      <line x1="71" y1="130" x2="99" y2="130" />
      <text class="title" x="85" y="48">反应器</text>
    </g>

    <g class="hot" :class="hotClass(9)" transform="translate(348,96)" @pointerdown="onHotDown($event, 9)">
      <title>{{ desc(9) }}</title>
      <rect class="hit" x="0" y="0" width="86" height="52" />
      <AnalogBar
        :width="10"
        :height="48"
        :value="num(9)"
        v-bind="bar(9)"
        :setpoint="setpointOf(9)"
        :status="st(9)"
      />
      <text class="lbl" x="16" y="14">{{ tag(9) }}</text>
      <text class="val" :class="valClass(9)" x="16" y="30">{{ m(9) }}</text>
      <text class="u" x="16" y="44">°C</text>
    </g>
    <g class="hot" :class="hotClass(7)" transform="translate(348,154)" @pointerdown="onHotDown($event, 7)">
      <title>{{ desc(7) }}</title>
      <rect class="hit" x="0" y="0" width="86" height="52" />
      <AnalogBar
        :width="10"
        :height="48"
        :value="num(7)"
        v-bind="bar(7)"
        :setpoint="setpointOf(7)"
        :status="st(7)"
      />
      <text class="lbl" x="16" y="14">{{ tag(7) }}</text>
      <text class="val" :class="valClass(7)" x="16" y="30">{{ m(7) }}</text>
      <text class="u" x="16" y="44">kPa</text>
    </g>
    <g class="hot" :class="hotClass(8)" transform="translate(348,212)" @pointerdown="onHotDown($event, 8)">
      <title>{{ desc(8) }}</title>
      <rect class="hit" x="0" y="0" width="86" height="52" />
      <AnalogBar
        :width="10"
        :height="48"
        :value="num(8)"
        v-bind="bar(8)"
        :setpoint="setpointOf(8)"
        :status="st(8)"
      />
      <text class="lbl" x="16" y="14">{{ tag(8) }}</text>
      <text class="val" :class="valClass(8)" x="16" y="30">{{ m(8) }}</text>
      <text class="u" x="16" y="44">%</text>
    </g>
    <g class="hot" :class="hotClass(21)" transform="translate(196,296)" @pointerdown="onHotDown($event, 21)">
      <title>{{ desc(21) }}</title>
      <rect class="hit" x="0" y="0" width="130" height="22" />
      <text class="small" x="0" y="14">{{ tag(21) }} {{ m(21) }}</text>
    </g>

    <g class="equip" transform="translate(430,28)">
      <rect x="0" y="0" width="150" height="70" />
      <line x1="18" y1="18" x2="132" y2="18" />
      <line x1="18" y1="35" x2="132" y2="35" />
      <line x1="18" y1="52" x2="132" y2="52" />
      <text class="title" x="75" y="44">冷凝器</text>
    </g>
    <g class="hot" :class="hotClass(22)" transform="translate(448,104)" @pointerdown="onHotDown($event, 22)">
      <title>{{ desc(22) }}</title>
      <rect class="hit" x="0" y="0" width="130" height="20" />
      <text class="small" x="0" y="14">{{ tag(22) }} {{ m(22) }}</text>
    </g>

    <g class="equip" transform="translate(640,58)">
      <rect x="0" y="0" width="180" height="160" rx="4" />
      <line x1="16" y1="78" x2="164" y2="78" stroke-dasharray="6 4" />
      <text class="title" x="90" y="24">气液分离器</text>
    </g>
    <g class="hot" :class="hotClass(11)" transform="translate(650,118)" @pointerdown="onHotDown($event, 11)">
      <title>{{ desc(11) }}</title>
      <rect class="hit" x="0" y="0" width="100" height="20" />
      <text class="val mid" :class="valClass(11)" x="50" y="14">{{ tag(11) }} {{ m(11) }}</text>
    </g>
    <g class="hot" :class="hotClass(13)" transform="translate(650,140)" @pointerdown="onHotDown($event, 13)">
      <title>{{ desc(13) }}</title>
      <rect class="hit" x="0" y="0" width="100" height="20" />
      <text class="val mid" :class="valClass(13)" x="50" y="14">{{ tag(13) }} {{ m(13) }}</text>
    </g>
    <g class="hot" :class="hotClass(12)" transform="translate(828,100)" @pointerdown="onHotDown($event, 12)">
      <title>{{ desc(12) }}</title>
      <rect class="hit" x="0" y="0" width="96" height="52" />
      <AnalogBar
        :width="10"
        :height="48"
        :value="num(12)"
        v-bind="bar(12)"
        :setpoint="setpointOf(12)"
        :status="st(12)"
      />
      <text class="lbl" x="16" y="14">{{ tag(12) }}</text>
      <text class="val" :class="valClass(12)" x="16" y="30">{{ m(12) }}</text>
      <text class="u" x="16" y="44">%</text>
    </g>

    <g class="equip" transform="translate(430,238)">
      <polygon points="20,20 130,50 20,80" />
      <text class="title" x="86" y="104">循环压缩机</text>
    </g>
    <g class="hot" :class="hotClass(20)" transform="translate(456,348)" @pointerdown="onHotDown($event, 20)">
      <title>{{ desc(20) }}</title>
      <rect class="hit" x="0" y="0" width="120" height="22" />
      <text class="val mid" :class="valClass(20)" x="50" y="16">{{ tag(20) }} {{ m(20) }}</text>
    </g>
    <g class="hot" :class="hotClass(5)" transform="translate(456,370)" @pointerdown="onHotDown($event, 5)">
      <title>{{ desc(5) }}</title>
      <rect class="hit" x="0" y="0" width="120" height="20" />
      <text class="small mid" x="50" y="14">{{ tag(5) }} {{ m(5) }}</text>
    </g>

    <g class="equip" transform="translate(880,148)">
      <rect x="20" y="0" width="120" height="250" />
      <path d="M32 40 H128 M32 80 H128 M32 120 H128 M32 160 H128 M32 200 H128" />
      <text class="title" x="80" y="22">汽提塔</text>
    </g>
    <g class="hot" :class="hotClass(18)" transform="translate(1018,210)" @pointerdown="onHotDown($event, 18)">
      <title>{{ desc(18) }}</title>
      <rect class="hit" x="0" y="0" width="90" height="22" />
      <text class="val" :class="valClass(18)" x="0" y="16">{{ tag(18) }} {{ m(18) }}</text>
    </g>
    <g class="hot" :class="hotClass(15)" transform="translate(1018,236)" @pointerdown="onHotDown($event, 15)">
      <title>{{ desc(15) }}</title>
      <rect class="hit" x="0" y="0" width="90" height="52" />
      <AnalogBar
        :width="10"
        :height="48"
        :value="num(15)"
        v-bind="bar(15)"
        :setpoint="setpointOf(15)"
        :status="st(15)"
      />
      <text class="lbl" x="16" y="14">{{ tag(15) }}</text>
      <text class="val" :class="valClass(15)" x="16" y="30">{{ m(15) }}</text>
      <text class="u" x="16" y="44">%</text>
    </g>
    <g class="hot" :class="hotClass(17)" transform="translate(900,412)" @pointerdown="onHotDown($event, 17)">
      <title>{{ desc(17) }}</title>
      <rect class="hit" x="0" y="0" width="140" height="22" />
      <text class="small mid" x="70" y="16">{{ tag(17) }} {{ m(17) }}</text>
    </g>

    <g class="stream">
      <path d="M340 170 H430 V63 H580" marker-end="url(#arr)" />
      <path d="M580 63 H640" marker-end="url(#arr)" />
      <path d="M820 98 H940 V148" marker-end="url(#arr)" />
      <path d="M820 188 H900" marker-end="url(#arr)" />
      <path d="M730 218 V288 H560" marker-end="url(#arr)" />
      <path d="M430 278 H340 V208" marker-end="url(#arr)" />
      <path d="M820 78 H1040" marker-end="url(#arr)" />
      <path d="M940 398 H1040" marker-end="url(#arr)" />
    </g>
    <g class="hot" :class="hotClass(6)" transform="translate(470,186)" @pointerdown="onHotDown($event, 6)">
      <title>{{ desc(6) }}</title>
      <rect class="hit" x="0" y="0" width="90" height="20" />
      <text class="small" x="0" y="14">{{ tag(6) }} {{ m(6) }}</text>
    </g>
    <g class="hot" :class="hotClass(10)" transform="translate(960,62)" @pointerdown="onHotDown($event, 10)">
      <title>{{ desc(10) }}</title>
      <rect class="hit" x="0" y="0" width="140" height="36" />
      <text x="0" y="14">{{ tag(10) }}</text>
      <text class="val" :class="valClass(10)" x="0" y="30">{{ m(10) }}</text>
    </g>
    <g class="hot" :class="hotClass(40)" transform="translate(960,430)" @pointerdown="onHotDown($event, 40)">
      <title>{{ desc(40) }}</title>
      <rect class="hit" x="0" y="0" width="150" height="28" />
      <AnalogBar
        :width="48"
        :height="8"
        :vertical="false"
        :value="num(40)"
        v-bind="bar(40)"
        :status="st(40)"
      />
      <text class="val" :class="valClass(40)" x="54" y="8">{{ tag(40) }} {{ m(40) }}</text>
    </g>
    <g class="hot" :class="hotClass(41)" transform="translate(960,458)" @pointerdown="onHotDown($event, 41)">
      <title>{{ desc(41) }}</title>
      <rect class="hit" x="0" y="0" width="150" height="28" />
      <AnalogBar
        :width="48"
        :height="8"
        :vertical="false"
        :value="num(41)"
        v-bind="bar(41)"
        :status="st(41)"
      />
      <text class="val" :class="valClass(41)" x="54" y="8">{{ tag(41) }} {{ m(41) }}</text>
    </g>

    <g class="valve">
      <polygon points="150,150 162,142 174,150 162,158" />
      <polygon points="900,388 912,380 924,388 912,396" />
    </g>

    <text class="foot" x="24" y="506">{{ status }} · 拖入趋势</text>
  </svg>
</template>

<script>
import AnalogBar from "./AnalogBar.vue";
import { LIMITS, exceptionStatus, formatPv } from "../limits.js";

export default {
  components: { AnalogBar },
  props: {
    xmeas: { type: Array, default: () => [] },
    meta: { type: Object, default: () => ({}) },
    status: { type: String, default: "未运行" },
    pinned: { type: Array, default: () => [] },
    focusKey: { type: String, default: "" },
    pens: { type: Object, default: () => ({}) },
    setpoints: { type: Object, default: () => ({}) },
  },
  emits: ["drag-tag", "focus-tag"],
  data() {
    return { pending: null };
  },
  beforeUnmount() {
    this.clearPending();
  },
  methods: {
    key(n) {
      return `xmeas:${n}`;
    },
    tag(n) {
      return this.meta[n]?.tag || `Y${n}`;
    },
    desc(n) {
      return this.meta[n]?.name_zh || this.tag(n);
    },
    num(n) {
      const v = this.xmeas[n - 1];
      return v == null ? null : Number(v);
    },
    m(n) {
      return formatPv(this.num(n));
    },
    st(n) {
      return exceptionStatus(n, this.num(n));
    },
    bar(n) {
      const lim = LIMITS[n] || { scaleLo: 0, scaleHi: 100 };
      return {
        scaleLo: lim.scaleLo,
        scaleHi: lim.scaleHi,
        loOp: lim.loOp ?? null,
        hiOp: lim.hiOp ?? null,
      };
    },
    setpointOf(n) {
      const v = this.setpoints[n];
      return v == null ? null : Number(v);
    },
    hotClass(n) {
      const k = this.key(n);
      return {
        pinned: this.pinned.includes(k),
        focus: this.focusKey === k,
        advisory: this.st(n) === "advisory",
        trip: this.st(n) === "trip",
      };
    },
    valClass(n) {
      return this.st(n);
    },
    onHotDown(ev, n) {
      if (ev.button != null && ev.button !== 0) return;
      this.clearPending();
      const key = this.key(n);
      this.pending = { key, x: ev.clientX, y: ev.clientY, pinned: this.pinned.includes(key) };
      window.addEventListener("pointermove", this.onWinMove, true);
      window.addEventListener("pointerup", this.onWinUp, true);
      window.addEventListener("pointercancel", this.onWinUp, true);
    },
    onWinMove(ev) {
      const p = this.pending;
      if (!p) return;
      if (Math.hypot(ev.clientX - p.x, ev.clientY - p.y) < 8) return;
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
.pfd {
  width: 100%;
  height: 100%;
  display: block;
  max-height: 100%;
}
.stream path {
  fill: none;
  stroke: #6a737b;
  stroke-width: 1.8;
}
.stream text,
.hot text {
  fill: var(--ink);
  font-family: var(--type-data);
  font-size: 10px;
}
.hot .mid {
  text-anchor: middle;
}
.equip rect,
.equip polygon {
  fill: var(--equip);
  stroke: #6a737b;
  stroke-width: 1.4;
}
.equip line,
.equip path,
.equip circle {
  fill: none;
  stroke: #6a737b;
  stroke-width: 1.1;
}
.title {
  font-family: var(--type-ui);
  font-size: 13px;
  text-anchor: middle;
  font-weight: 600;
  fill: var(--ink);
}
.lbl,
.u {
  font-size: 9px;
  fill: var(--ink-soft);
  font-family: var(--type-data);
}
.val {
  font-family: var(--type-data);
  font-size: 12px;
  font-weight: 500;
  fill: var(--ink);
}
.val.advisory {
  fill: var(--advisory);
  font-weight: 600;
}
.val.trip {
  fill: var(--trip);
  font-weight: 700;
}
.small {
  font-family: var(--type-data);
  font-size: 10px;
  fill: var(--ink-soft);
}
.hot .hit {
  fill: transparent;
  cursor: grab;
}
.hot.pinned .hit {
  fill: var(--event);
  opacity: 0.08;
}
.hot.focus .hit {
  fill: var(--event);
  opacity: 0.14;
}
.hot:hover .hit {
  fill: #2c3338;
  opacity: 0.06;
}
.valve polygon {
  fill: var(--panel);
  stroke: #6a737b;
  stroke-width: 1.2;
}
.foot {
  font-size: 11px;
  fill: var(--ink-soft);
}
</style>
