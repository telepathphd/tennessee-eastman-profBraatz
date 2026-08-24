<template>
  <svg class="pfd" viewBox="0 0 1120 560" role="img" aria-label="Tennessee Eastman 工艺流程图">
    <defs>
      <pattern id="sheet" width="16" height="16" patternUnits="userSpaceOnUse">
        <path d="M16 0H0V16" fill="none" stroke="#c5d3db" stroke-width="0.6" />
      </pattern>
      <marker id="arr" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto">
        <path d="M0 0 L10 5 L0 10 z" fill="#1a5f8a" />
      </marker>
    </defs>
    <rect width="1120" height="560" fill="#e8eef1" />
    <rect width="1120" height="560" fill="url(#sheet)" />

    <!-- feeds -->
    <g class="stream">
      <path d="M40 120 H170" marker-end="url(#arr)" />
      <path d="M40 160 H170" marker-end="url(#arr)" />
      <path d="M40 200 H170" marker-end="url(#arr)" />
      <path d="M40 240 H170" marker-end="url(#arr)" />
      <text x="44" y="108">S1 A</text>
      <text x="44" y="148">S2 D</text>
      <text x="44" y="188">S3 E</text>
      <text x="44" y="228">S4 A/C</text>
      <text class="val" x="86" y="108">{{ m(1) }}</text>
      <text class="val" x="86" y="148">{{ m(2) }}</text>
      <text class="val" x="86" y="188">{{ m(3) }}</text>
      <text class="val" x="86" y="228">{{ m(4) }}</text>
    </g>

    <!-- reactor -->
    <g class="unit" transform="translate(170,80)">
      <rect x="0" y="20" width="170" height="200" rx="86" />
      <line x1="20" y1="70" x2="150" y2="70" />
      <circle cx="85" cy="130" r="14" />
      <line x1="85" y1="116" x2="85" y2="144" />
      <line x1="71" y1="130" x2="99" y2="130" />
      <text class="title" x="85" y="48">反应器</text>
      <text class="val" x="85" y="188">{{ m(9) }} °C</text>
      <text class="val" x="85" y="206">{{ m(7) }} kPa</text>
      <text class="val" x="85" y="224">L {{ m(8) }} %</text>
      <text class="small" x="85" y="248">CW out {{ m(21) }} °C</text>
    </g>

    <!-- condenser -->
    <g class="unit" transform="translate(430,40)">
      <rect x="0" y="0" width="150" height="70" />
      <line x1="18" y1="18" x2="132" y2="18" />
      <line x1="18" y1="35" x2="132" y2="35" />
      <line x1="18" y1="52" x2="132" y2="52" />
      <text class="title" x="75" y="46">冷凝器</text>
      <text class="small" x="75" y="88">CW out {{ m(22) }} °C</text>
    </g>

    <!-- separator -->
    <g class="unit" transform="translate(640,70)">
      <rect x="0" y="0" width="180" height="160" rx="8" />
      <line x1="16" y1="78" x2="164" y2="78" stroke-dasharray="6 4" />
      <text class="title" x="90" y="28">气液分离器</text>
      <text class="val" x="90" y="108">{{ m(11) }} °C</text>
      <text class="val" x="90" y="126">{{ m(13) }} kPa</text>
      <text class="val" x="90" y="144">L {{ m(12) }} %</text>
    </g>

    <!-- compressor -->
    <g class="unit" transform="translate(430,250)">
      <polygon points="20,20 130,50 20,80" />
      <text class="title" x="86" y="108">循环压缩机</text>
      <text class="val" x="86" y="126">{{ m(20) }} kW</text>
      <text class="small" x="86" y="144">S8 {{ m(5) }}</text>
    </g>

    <!-- stripper -->
    <g class="unit" transform="translate(880,160)">
      <rect x="20" y="0" width="120" height="250" />
      <path d="M32 40 H128 M32 80 H128 M32 120 H128 M32 160 H128 M32 200 H128" />
      <text class="title" x="80" y="24">汽提塔</text>
      <text class="val" x="80" y="268">{{ m(18) }} °C</text>
      <text class="val" x="80" y="286">L {{ m(15) }} %</text>
      <text class="small" x="80" y="304">S11 {{ m(17) }}</text>
    </g>

    <!-- process lines -->
    <g class="stream">
      <path d="M340 180 H430 V75 H580" marker-end="url(#arr)" />
      <path d="M580 75 H640" marker-end="url(#arr)" />
      <path d="M820 110 H940 V160" marker-end="url(#arr)" />
      <path d="M820 200 H900" marker-end="url(#arr)" />
      <path d="M730 230 V300 H560" marker-end="url(#arr)" />
      <path d="M430 290 H340 V220" marker-end="url(#arr)" />
      <path d="M820 90 H1040" marker-end="url(#arr)" />
      <path d="M940 410 H1040" marker-end="url(#arr)" />
      <text x="980" y="78">S9 放空</text>
      <text class="val" x="980" y="96">{{ m(10) }}</text>
      <text x="980" y="430">S11 产品</text>
      <text class="val" x="980" y="448">G {{ m(40) }} %</text>
      <text class="val" x="980" y="466">H {{ m(41) }} %</text>
      <text x="500" y="200">S6 {{ m(6) }}</text>
    </g>

    <!-- valves as small diamonds on feeds / product -->
    <g class="valve">
      <polygon points="150,160 162,152 174,160 162,168" />
      <polygon points="900,400 912,392 924,400 912,408" />
    </g>

    <g class="title-block">
      <rect x="24" y="500" width="420" height="44" />
      <text class="tb-k" x="36" y="518">DRAWING</text>
      <text class="tb-v" x="120" y="518">TE-PFD-01  Downs &amp; Vogel 1993</text>
      <text class="tb-k" x="36" y="536">STATUS</text>
      <text class="tb-v" x="120" y="536">{{ status }}</text>
    </g>
  </svg>
</template>

<script>
export default {
  props: {
    xmeas: { type: Array, default: () => [] },
    status: { type: String, default: "未运行" },
  },
  methods: {
    m(n) {
      const v = this.xmeas[n - 1];
      if (v == null || Number.isNaN(v)) return "—";
      const abs = Math.abs(v);
      if (abs >= 100) return v.toFixed(1);
      if (abs >= 10) return v.toFixed(2);
      return v.toFixed(3);
    },
  },
};
</script>

<style scoped>
.pfd {
  width: 100%;
  height: auto;
  display: block;
  border: 1px solid var(--rule);
}
.stream path {
  fill: none;
  stroke: var(--line);
  stroke-width: 2.2;
}
.stream text,
.unit text {
  fill: var(--ink);
  font-family: var(--type-ui);
  font-size: 11px;
}
.unit rect,
.unit polygon {
  fill: var(--equip);
  stroke: var(--ink);
  stroke-width: 1.6;
}
.unit line,
.unit path,
.unit circle {
  fill: none;
  stroke: var(--ink);
  stroke-width: 1.2;
}
.title {
  font-family: var(--type-display);
  font-size: 14px;
  text-anchor: middle;
  font-weight: 600;
}
.val,
.small {
  font-family: var(--type-data);
  text-anchor: middle;
}
.val {
  font-size: 12px;
  font-weight: 600;
  fill: var(--ink);
}
.small {
  font-size: 10px;
  fill: var(--ink-soft);
}
.valve polygon {
  fill: var(--sheet);
  stroke: var(--ink);
  stroke-width: 1.4;
}
.title-block rect {
  fill: #f7fafb;
  stroke: var(--ink);
}
.tb-k {
  font-size: 9px;
  letter-spacing: 0.12em;
  fill: var(--ink-soft);
}
.tb-v {
  font-family: var(--type-data);
  font-size: 12px;
}
</style>
