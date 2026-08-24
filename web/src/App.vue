<template>
  <div class="console">
    <header class="mast">
      <div class="brand">
        <span class="mark">TE</span>
        <span class="loop">{{ mode === "closed_loop" ? "闭环" : "开环" }}</span>
      </div>
      <dl class="status">
        <div>
          <dt>时刻</dt>
          <dd>{{ (cursorT / 3600).toFixed(3) }} h</dd>
        </div>
        <div>
          <dt>联锁</dt>
          <dd :class="{ trip: shutdownNow }">{{ interlockLabel }}</dd>
        </div>
        <div>
          <dt>注入</dt>
          <dd>{{ activeInjLabel }}</dd>
        </div>
      </dl>
      <button class="go" type="button" :disabled="busy" @click="simulate">
        {{ busy ? `积分中 ${elapsed} s` : "运行仿真" }}
      </button>
    </header>

    <p v-if="error" class="banner trip">{{ error }}</p>
    <p v-else-if="result?.shutdown" class="banner trip">
      联锁动作
      <template v-if="result.shutdown_time_s != null">
        @ {{ (result.shutdown_time_s / 3600).toFixed(3) }} h
      </template>
      · {{ (result.shutdown_reasons || []).join("；") || "见过程约束" }}
    </p>
    <p v-else-if="busy" class="banner">正在积分 {{ hoursTotal }} h · {{ elapsed }} s</p>

    <aside class="run">
      <h2>运行</h2>
      <label>
        回路
        <select v-model="mode">
          <option value="closed_loop">闭环（temain_mod）</option>
          <option value="open_loop">开环（temain）</option>
        </select>
      </label>
      <div class="row">
        <label>
          时长 / h
          <input v-model.number="hoursTotal" type="number" min="0.1" max="96" step="0.5" />
        </label>
        <label>
          记录间隔 / s
          <input v-model.number="recordEvery" type="number" min="1" max="3600" />
        </label>
      </div>
      <label>
        随机种子
        <input v-model.number="seed" type="number" />
      </label>
      <div class="presets">
        <button type="button" @click="applyPreset(1, 0.25)">预览 1 h</button>
        <button type="button" @click="applyPreset(8, 2)">短跑 8 h</button>
        <button type="button" @click="applyPreset(48, 8)">标准 48 h</button>
      </div>

      <h3>扰动注入</h3>
      <p class="hint">勾选后在该时刻打开扰动。右侧为开始时刻（小时）。</p>
      <div v-for="group in idvGroups" :key="group.kind" class="idv-group">
        <h4>{{ group.kind }}</h4>
        <label v-for="d in group.items" :key="d.n" class="idv">
          <input v-model="idvOn[d.n]" type="checkbox" />
          <span class="idv-n">{{ String(d.n).padStart(2, "0") }}</span>
          <span class="idv-body">{{ d.name_zh }}</span>
          <input
            v-model.number="idvHour[d.n]"
            class="hour"
            type="number"
            min="0"
            step="0.5"
            :disabled="!idvOn[d.n]"
          />
        </label>
      </div>

      <template v-if="mode === 'closed_loop'">
        <h3>设定值</h3>
        <p class="hint">「锁」阻止级联外环改写该设定值。</p>
        <label v-for="s in visibleSetpoints" :key="s.n" class="sp">
          <span>
            <button type="button" class="lock" :class="{ on: held.has(s.n) }" @click="toggleHold(s.n)">
              {{ held.has(s.n) ? "锁" : "随" }}
            </button>
            {{ s.n }}. {{ s.name_zh }}
            <small>{{ s.unit }}</small>
          </span>
          <input v-model.number="setpoints[s.n]" type="number" step="any" />
        </label>
        <button type="button" class="link" @click="showAllSp = !showAllSp">
          {{ showAllSp ? "只看装置设定" : "显示内环 / 备用" }}
        </button>
      </template>
      <template v-else>
        <h3>开环阀位</h3>
        <label>
          汽提塔液位设定 / %
          <input v-model.number="stripperSp" type="number" step="0.1" />
        </label>
        <label v-for="v in catalog?.xmv || []" :key="v.n" class="sp">
          <span>{{ v.n }}. {{ v.name_zh }}</span>
          <input v-model.number="openXmv[v.n]" type="number" step="0.1" />
        </label>
      </template>
    </aside>

    <section class="stage">
      <PlantPfd
        :xmeas="frameXmeas"
        :status="pfdStatus"
        :pinned="tags"
        :focus-key="focusKey"
        :pens="penMap"
        :setpoints="pvSetpoints"
        @pin="onPin"
      />
    </section>

    <aside class="kpis">
      <AnalogRail :items="kpiItems" @pin="onPin" />
    </aside>

    <section class="trends">
      <TrendChart
        :time-s="result?.time_s || []"
        :layers="chartLayers"
        :cursor="cursor"
        :injections="result?.injections || []"
        :shutdown-time-s="result?.shutdown_time_s ?? null"
        :focus-key="focusKey"
        @update:cursor="cursor = $event"
        @toggle-hidden="toggleHidden"
        @remove="removeTag"
      />
      <div class="picker">
        <input v-model="query" type="search" placeholder="补充趋势：反应器、液位、产品…" />
        <button v-for="opt in pickerOpts" :key="opt.key" type="button" @click="addTag(opt.key)">
          {{ opt.label }}
        </button>
      </div>
    </section>
  </div>
</template>

<script>
import { loadCatalog, runSim } from "./api.js";
import AnalogRail from "./components/AnalogRail.vue";
import PlantPfd from "./components/PlantPfd.vue";
import TrendChart from "./components/TrendChart.vue";
import { KPI_STRIP, LIMITS, exceptionStatus, formatPv } from "./limits.js";

const PENS = ["#3d6b99", "#5a6b4f", "#8a5a2b", "#5c4e78", "#3b5368", "#7a4040", "#4a6670", "#6b5a3a"];
const STORE = "te-console-v1";
const IDV_ORDER = ["阶跃", "随机变化", "缓慢漂移", "卡涩", "未知"];
const MAX_TAGS = 8;

export default {
  components: { AnalogRail, PlantPfd, TrendChart },
  data() {
    return {
      catalog: null,
      mode: "closed_loop",
      hoursTotal: 8,
      recordEvery: 60,
      seed: 0,
      idvOn: {},
      idvHour: {},
      setpoints: {},
      held: new Set(),
      showAllSp: false,
      openXmv: {},
      stripperSp: 50,
      tags: ["xmeas:7", "xmeas:8", "xmeas:9", "xmeas:17", "xmeas:40"],
      hiddenTags: [],
      focusKey: "",
      query: "",
      result: null,
      cursor: 0,
      busy: false,
      error: "",
      elapsed: "0.0",
    };
  },
  computed: {
    visibleSetpoints() {
      const all = this.catalog?.setpoints || [];
      if (this.showAllSp) return all;
      return all.filter((s) => s.group_zh === "装置");
    },
    cursorT() {
      return this.result?.time_s?.[this.cursor] ?? 0;
    },
    shutdownNow() {
      if (!this.result?.shutdown) return false;
      return this.cursorT >= (this.result.shutdown_time_s ?? 0);
    },
    interlockLabel() {
      if (!this.result) return "—";
      return this.shutdownNow ? "已停车" : "运行窗";
    },
    frameXmeas() {
      const x = this.result?.xmeas;
      if (!x) return [];
      return x.map((col) => col[this.cursor]);
    },
    activeInjections() {
      return (this.result?.injections || []).filter((inj) => this.cursorT >= inj.start_step);
    },
    activeInjLabel() {
      const on = this.activeInjections;
      if (!on.length) return "无";
      return on.map((inj) => `IDV${inj.idv}`).join(" ");
    },
    pfdStatus() {
      if (!this.result) return "未运行";
      if (this.shutdownNow) return "联锁停车";
      const on = this.activeInjections.map((inj) => `IDV${inj.idv}`);
      return on.length ? `注入 ${on.join(" ")}` : "稳态 / 未注入";
    },
    idvGroups() {
      const items = this.catalog?.idv || [];
      const buckets = {};
      for (const d of items) {
        const k = d.kind_zh || "未知";
        (buckets[k] ||= []).push(d);
      }
      return IDV_ORDER.filter((k) => buckets[k]).map((kind) => ({ kind, items: buckets[kind] }));
    },
    penMap() {
      const map = {};
      this.tags.forEach((key, i) => {
        map[key] = PENS[i % PENS.length];
      });
      return map;
    },
    pvSetpoints() {
      const map = {};
      if (this.mode !== "closed_loop") return map;
      for (const s of this.catalog?.setpoints || []) {
        const m = String(s.pv || "").match(/XMEAS\((\d+)\)/i);
        if (m) map[Number(m[1])] = this.setpoints[s.n];
      }
      return map;
    },
    kpiItems() {
      return KPI_STRIP.map((k) => {
        const key = `xmeas:${k.n}`;
        const meta = this.catalog?.xmeas?.[k.n - 1];
        const lim = LIMITS[k.n] || { scaleLo: 0, scaleHi: 100 };
        const value = this.frameXmeas[k.n - 1];
        const status = exceptionStatus(k.n, value);
        return {
          key,
          name: k.name,
          unit: meta?.unit || "",
          value,
          text: formatPv(value),
          status,
          pinned: this.tags.includes(key),
          focus: this.focusKey === key,
          scaleLo: lim.scaleLo,
          scaleHi: lim.scaleHi,
          loOp: lim.loOp ?? null,
          hiOp: lim.hiOp ?? null,
          setpoint: this.pvSetpoints[k.n] ?? null,
          hint: "点击钉到趋势，Shift+点击移除",
        };
      });
    },
    chartLayers() {
      return this.tags.map((key, i) => {
        const { kind, n } = splitTag(key);
        const meta = kind === "xmeas" ? this.catalog?.xmeas?.[n - 1] : this.catalog?.xmv?.[n - 1];
        const values = kind === "xmeas" ? this.result?.xmeas?.[n - 1] : this.result?.xmv?.[n - 1];
        const lim = kind === "xmeas" ? LIMITS[n] : null;
        return {
          key,
          color: PENS[i % PENS.length],
          label: meta ? `${kind === "xmeas" ? "Y" : "U"}${n} ${meta.name_zh}` : key,
          unit: meta?.unit || "",
          values: values || [],
          hidden: this.hiddenTags.includes(key),
          loOp: lim?.loOp ?? null,
          hiOp: lim?.hiOp ?? null,
          loShutdown: lim?.loShutdown ?? null,
          hiShutdown: lim?.hiShutdown ?? null,
        };
      });
    },
    pickerOpts() {
      if (!this.catalog || !this.query.trim()) return [];
      const q = this.query.trim().toLowerCase();
      const out = [];
      for (const m of this.catalog.xmeas) {
        const key = `xmeas:${m.n}`;
        const label = `Y${m.n} ${m.name_zh}`;
        if (!this.tags.includes(key) && matches(label, m.n, q)) out.push({ key, label });
      }
      for (const m of this.catalog.xmv) {
        const key = `xmv:${m.n}`;
        const label = `U${m.n} ${m.name_zh}`;
        if (!this.tags.includes(key) && matches(label, m.n, q)) out.push({ key, label });
      }
      return out.slice(0, 12);
    },
  },
  async mounted() {
    this.restore();
    try {
      this.catalog = await loadCatalog();
    } catch (err) {
      this.error = `无法读取目录：${err.message}。请先启动 rust 侧 te-console。`;
      return;
    }
    if (!this.seed) this.seed = this.catalog.default_seed;
    for (const s of this.catalog.setpoints) {
      if (this.setpoints[s.n] == null) this.setpoints[s.n] = s.default;
    }
    for (const d of this.catalog.idv) {
      if (this.idvHour[d.n] == null) this.idvHour[d.n] = 2;
      if (this.idvOn[d.n] == null) this.idvOn[d.n] = d.n === 12;
    }
    for (const v of this.catalog.xmv) {
      if (this.openXmv[v.n] == null) this.openXmv[v.n] = v.n === 10 ? 38 : 50;
    }
  },
  methods: {
    applyPreset(hours, injectAt) {
      this.hoursTotal = hours;
      for (const n of Object.keys(this.idvHour)) this.idvHour[n] = injectAt;
      this.save();
    },
    toggleHold(n) {
      const next = new Set(this.held);
      if (next.has(n)) next.delete(n);
      else next.add(n);
      this.held = next;
      this.save();
    },
    onPin({ key, shift }) {
      if (shift) this.removeTag(key);
      else this.addTag(key);
    },
    addTag(key) {
      if (this.tags.includes(key)) {
        this.hiddenTags = this.hiddenTags.filter((k) => k !== key);
      } else if (this.tags.length < MAX_TAGS) {
        this.tags = [...this.tags, key];
      }
      this.hiddenTags = this.hiddenTags.filter((k) => k !== key);
      this.focusKey = key;
      this.query = "";
      this.save();
    },
    removeTag(key) {
      this.tags = this.tags.filter((k) => k !== key);
      this.hiddenTags = this.hiddenTags.filter((k) => k !== key);
      if (this.focusKey === key) this.focusKey = this.tags[0] || "";
      this.save();
    },
    toggleHidden(key) {
      if (this.hiddenTags.includes(key)) this.hiddenTags = this.hiddenTags.filter((k) => k !== key);
      else this.hiddenTags = [...this.hiddenTags, key];
      this.save();
    },
    save() {
      localStorage.setItem(
        STORE,
        JSON.stringify({
          mode: this.mode,
          hoursTotal: this.hoursTotal,
          recordEvery: this.recordEvery,
          seed: this.seed,
          idvOn: this.idvOn,
          idvHour: this.idvHour,
          setpoints: this.setpoints,
          held: [...this.held],
          tags: this.tags,
          hiddenTags: this.hiddenTags,
          stripperSp: this.stripperSp,
          openXmv: this.openXmv,
        })
      );
    },
    restore() {
      try {
        const raw = JSON.parse(localStorage.getItem(STORE) || "null");
        if (!raw) return;
        this.mode = raw.mode ?? this.mode;
        this.hoursTotal = raw.hoursTotal ?? this.hoursTotal;
        this.recordEvery = raw.recordEvery ?? this.recordEvery;
        this.seed = raw.seed ?? this.seed;
        this.idvOn = raw.idvOn ?? {};
        this.idvHour = raw.idvHour ?? {};
        this.setpoints = raw.setpoints ?? {};
        this.tags = raw.tags ?? this.tags;
        this.hiddenTags = raw.hiddenTags ?? [];
        this.stripperSp = raw.stripperSp ?? this.stripperSp;
        this.openXmv = raw.openXmv ?? {};
        this.held = new Set(raw.held || []);
      } catch {
        /* ignore broken local config */
      }
    },
    async simulate() {
      this.error = "";
      this.busy = true;
      const t0 = performance.now();
      const tick = setInterval(() => {
        this.elapsed = ((performance.now() - t0) / 1000).toFixed(1);
      }, 200);
      try {
        const npts = Math.max(1, Math.round(this.hoursTotal * 3600));
        const injections = Object.entries(this.idvOn)
          .filter(([, on]) => on)
          .map(([n]) => ({
            idv: Number(n),
            start_step: Math.max(1, Math.round((Number(this.idvHour[n]) || 0) * 3600)),
          }));
        const body = {
          mode: this.mode,
          npts,
          record_every: this.recordEvery,
          seed: this.seed,
          injections,
        };
        if (this.mode === "closed_loop") {
          body.setpoints = this.setpoints;
          body.held_setpoints = [...this.held];
        } else {
          body.open_loop_xmv = this.openXmv;
          body.open_loop_stripper_sp = this.stripperSp;
        }
        this.result = await runSim(body);
        this.cursor = Math.max(0, this.result.time_s.length - 1);
        this.save();
      } catch (err) {
        this.error = err.message || String(err);
      } finally {
        clearInterval(tick);
        this.busy = false;
      }
    },
  },
};

function splitTag(key) {
  const [kind, n] = key.split(":");
  return { kind, n: Number(n) };
}

function matches(label, n, q) {
  return label.toLowerCase().includes(q) || String(n) === q;
}
</script>

<style scoped>
.console {
  height: 100%;
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr) 168px;
  grid-template-rows: 40px minmax(0, auto) minmax(0, 1fr) minmax(220px, 38vh);
  background: var(--console);
  color: var(--ink);
}
.mast {
  grid-column: 1 / -1;
  grid-row: 1;
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0 0.7rem;
  background: #c8ccd0;
  border-bottom: 1px solid var(--rule);
}
.brand {
  display: flex;
  align-items: baseline;
  gap: 0.55rem;
  font-weight: 600;
}
.mark {
  font-size: 1.05rem;
  letter-spacing: 0.06em;
}
.loop {
  font-size: 0.82rem;
  color: var(--ink-soft);
}
.status {
  display: flex;
  gap: 1.2rem;
  margin: 0;
  flex: 1;
}
.status dt {
  font-size: 0.62rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-soft);
}
.status dd {
  margin: 0.05rem 0 0;
  font-family: var(--type-data);
  font-size: 0.95rem;
}
.trip {
  color: var(--trip);
}
.go {
  border: 1px solid var(--ink);
  background: var(--ink);
  color: var(--panel);
  padding: 0.28rem 0.8rem;
  cursor: pointer;
  font-size: 0.82rem;
}
.go:disabled {
  opacity: 0.55;
  cursor: wait;
}
.banner {
  grid-column: 1 / -1;
  grid-row: 2;
  margin: 0;
  padding: 0.35rem 0.7rem;
  background: #eceef0;
  border-bottom: 1px solid var(--rule);
  font-size: 0.82rem;
}
.banner.trip {
  background: #f0e4e4;
  color: var(--trip);
}
.run {
  grid-column: 1;
  grid-row: 3;
  overflow: auto;
  padding: 0.6rem 0.7rem 1rem;
  background: var(--panel);
  border-right: 1px solid var(--rule);
}
.stage {
  grid-column: 2;
  grid-row: 3;
  min-width: 0;
  min-height: 0;
  padding: 0.35rem;
  overflow: hidden;
}
.kpis {
  grid-column: 3;
  grid-row: 3;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.trends {
  grid-column: 1 / -1;
  grid-row: 4;
  min-height: 0;
  display: flex;
  flex-direction: column;
  border-top: 1px solid var(--rule);
  background: var(--panel);
}
.trends > .trend {
  flex: 1;
  min-height: 0;
}
h2,
h3,
h4 {
  margin: 0.15rem 0 0.35rem;
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-soft);
  font-weight: 600;
}
h3 {
  margin-top: 0.85rem;
}
h4 {
  margin: 0.35rem 0 0.15rem;
  text-transform: none;
  letter-spacing: 0;
  font-size: 0.72rem;
}
label {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  font-size: 0.78rem;
  color: var(--ink-soft);
  margin-bottom: 0.4rem;
}
input,
select {
  border: 1px solid var(--rule);
  background: #f3f5f6;
  padding: 0.28rem 0.4rem;
  color: var(--ink);
  font-family: var(--type-data);
}
.row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.4rem;
}
.presets,
.picker {
  display: flex;
  flex-wrap: wrap;
  gap: 0.3rem;
  margin: 0.25rem 0 0.5rem;
}
.presets button,
.picker button,
.link,
.lock {
  border: 1px solid var(--ink);
  background: transparent;
  color: var(--ink);
  padding: 0.2rem 0.45rem;
  cursor: pointer;
  font-size: 0.75rem;
}
.hint {
  font-size: 0.72rem;
  color: var(--ink-soft);
  margin: 0 0 0.35rem;
}
.idv {
  display: grid;
  grid-template-columns: auto auto 1fr 3.8rem;
  gap: 0.3rem;
  align-items: center;
  margin: 0 0 0.12rem;
  flex-direction: row;
}
.idv-n {
  font-family: var(--type-data);
  font-size: 0.72rem;
}
.idv-body {
  font-size: 0.74rem;
  color: var(--ink);
}
.hour {
  width: 3.8rem;
}
.sp {
  display: grid;
  grid-template-columns: 1fr 6.2rem;
  align-items: center;
  gap: 0.35rem;
  margin-bottom: 0.28rem;
  flex-direction: row;
}
.sp span {
  display: flex;
  flex-wrap: wrap;
  gap: 0.2rem 0.35rem;
  align-items: baseline;
  color: var(--ink);
}
.sp small {
  color: var(--ink-soft);
}
.lock.on {
  background: var(--ink);
  color: var(--panel);
}
.link {
  margin-bottom: 0.6rem;
}
.picker {
  padding: 0.25rem 0.5rem 0.45rem;
}
.picker input {
  flex: 1 1 12rem;
}
@media (max-width: 880px) {
  .console {
    height: auto;
    min-height: 100%;
    grid-template-columns: 1fr;
    grid-template-rows: auto;
  }
  .mast,
  .banner,
  .run,
  .stage,
  .kpis,
  .trends {
    grid-column: 1;
    grid-row: auto;
  }
  .run {
    max-height: 42vh;
    border-right: 0;
    border-bottom: 1px solid var(--rule);
  }
  .stage {
    min-height: 320px;
  }
  .trends {
    height: 420px;
  }
}
</style>
