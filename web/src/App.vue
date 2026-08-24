<template>
  <div class="desk">
    <header class="mast">
      <div>
        <p class="eyebrow">Tennessee Eastman Process · 本地操作台</p>
        <h1>工艺图纸与记录纸</h1>
      </div>
      <dl class="clock">
        <div>
          <dt>仿真时刻</dt>
          <dd>{{ (cursorT / 3600).toFixed(3) }} h</dd>
        </div>
        <div>
          <dt>回路</dt>
          <dd>{{ mode === "closed_loop" ? "全厂闭环" : "开环 + 汽提液位" }}</dd>
        </div>
        <div>
          <dt>联锁</dt>
          <dd :class="{ trip: result?.shutdown }">{{ result?.shutdown ? "已停车" : "运行窗" }}</dd>
        </div>
      </dl>
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

    <div class="layout">
      <aside class="log">
        <h2>运行单</h2>
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
        <p class="hint">勾选后在该时刻打开 IDV。右侧为开始时刻（小时）。</p>
        <div class="idv-list">
          <label v-for="d in catalog?.idv || []" :key="d.n" class="idv">
            <input v-model="idvOn[d.n]" type="checkbox" />
            <span class="idv-n">{{ String(d.n).padStart(2, "0") }}</span>
            <span class="idv-body">
              {{ d.name_zh }}
              <em>{{ d.kind_zh }}</em>
            </span>
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
          <p class="hint">「锁」阻止级联外环改写该 SETPT。</p>
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

        <button class="run" type="button" :disabled="busy" @click="simulate">
          {{ busy ? "积分中…" : "运行仿真" }}
        </button>
      </aside>

      <main class="sheet">
        <PlantPfd :xmeas="frameXmeas" :status="pfdStatus" />
        <TrendChart
          :time-s="result?.time_s || []"
          :series="chartSeries"
          :cursor="cursor"
          :injections="result?.injections || []"
          :shutdown-time-s="result?.shutdown_time_s ?? null"
          @update:cursor="cursor = $event"
        />
        <div class="legend">
          <label v-for="tag in selectedMeta" :key="tag.key" class="pen">
            <input checked type="checkbox" @change="removeTag(tag.key)" />
            <i :style="{ background: tag.color }" />
            {{ tag.label }}
            <b>{{ fmt(tag.value) }} {{ tag.unit }}</b>
          </label>
        </div>
        <div class="picker">
          <input v-model="query" type="search" placeholder="添加趋势：反应器、液位、产品…" />
          <button v-for="opt in pickerOpts" :key="opt.key" type="button" @click="addTag(opt.key)">
            {{ opt.label }}
          </button>
        </div>
        <input
          v-if="result"
          class="scrub"
          type="range"
          min="0"
          :max="Math.max(0, result.time_s.length - 1)"
          v-model.number="cursor"
        />
        <p class="foot">图纸数字跟随滑尺。配置只存在本机，不经过网络。</p>
      </main>
    </div>
  </div>
</template>

<script>
import { loadCatalog, runSim } from "./api.js";
import PlantPfd from "./components/PlantPfd.vue";
import TrendChart from "./components/TrendChart.vue";

const PENS = ["#1a5f8a", "#9a1f2e", "#2f6b4f", "#c47a12", "#5b3d8f", "#3b5368"];
const STORE = "te-console-v1";

export default {
  components: { PlantPfd, TrendChart },
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
    frameXmeas() {
      const x = this.result?.xmeas;
      if (!x) return [];
      return x.map((col) => col[this.cursor]);
    },
    pfdStatus() {
      if (!this.result) return "未运行";
      if (this.result.shutdown && this.cursorT >= (this.result.shutdown_time_s ?? 0)) return "联锁停车";
      const on = (this.result.injections || [])
        .filter((inj) => this.cursorT >= inj.start_step)
        .map((inj) => `IDV${inj.idv}`);
      return on.length ? `注入 ${on.join(" ")}` : "稳态 / 未注入";
    },
    selectedMeta() {
      return this.tags.map((key, i) => {
        const { kind, n } = splitTag(key);
        const meta = kind === "xmeas" ? this.catalog?.xmeas?.[n - 1] : this.catalog?.xmv?.[n - 1];
        const values = kind === "xmeas" ? this.result?.xmeas?.[n - 1] : this.result?.xmv?.[n - 1];
        return {
          key,
          color: PENS[i % PENS.length],
          label: meta ? `${kind === "xmeas" ? "Y" : "U"}${n} ${meta.name_zh}` : key,
          unit: meta?.unit || "",
          value: values?.[this.cursor],
          values: values || [],
        };
      });
    },
    chartSeries() {
      return this.selectedMeta.filter((s) => s.values.length).map((s) => ({ values: s.values }));
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
    fmt(v) {
      if (v == null || Number.isNaN(v)) return "—";
      return Number(v).toPrecision(5);
    },
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
    addTag(key) {
      if (!this.tags.includes(key)) this.tags = [...this.tags, key].slice(0, 6);
      this.query = "";
    },
    removeTag(key) {
      this.tags = this.tags.filter((k) => k !== key);
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
.desk {
  min-height: 100vh;
  padding: 1.25rem 1.5rem 2rem;
  background:
    radial-gradient(80% 50% at 50% 0%, #3a424a 0%, transparent 55%),
    var(--desk);
}
.mast {
  display: flex;
  justify-content: space-between;
  gap: 1.5rem;
  align-items: end;
  color: #e8eef1;
  margin-bottom: 1rem;
}
.eyebrow {
  margin: 0;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  font-size: 0.72rem;
  color: #b7c4ce;
}
h1 {
  margin: 0.2rem 0 0;
  font-family: var(--type-display);
  font-size: 2rem;
  font-weight: 600;
}
.clock {
  display: flex;
  gap: 1.5rem;
  margin: 0;
}
.clock dt {
  font-size: 0.7rem;
  letter-spacing: 0.08em;
  color: #b7c4ce;
}
.clock dd {
  margin: 0.15rem 0 0;
  font-family: var(--type-data);
  font-size: 1.05rem;
}
.trip {
  color: #f0b4b8;
}
.banner {
  background: #edf3ea;
  color: var(--ink);
  padding: 0.55rem 0.8rem;
  margin: 0 0 0.9rem;
  border-left: 4px solid var(--ok);
}
.banner.trip {
  background: #f6e4e6;
  border-left-color: var(--trip);
  color: var(--trip);
}
.layout {
  display: grid;
  grid-template-columns: minmax(280px, 340px) 1fr;
  gap: 1rem;
  align-items: start;
}
.log,
.sheet {
  background: var(--sheet);
  border: 1px solid #9aaeb8;
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.28);
}
.log {
  padding: 1rem;
  max-height: calc(100vh - 8rem);
  overflow: auto;
}
.sheet {
  padding: 0.85rem;
}
h2,
h3 {
  font-family: var(--type-display);
  margin: 0.4rem 0 0.5rem;
}
h2 {
  font-size: 1.25rem;
}
h3 {
  font-size: 1.05rem;
  margin-top: 1.1rem;
}
label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.82rem;
  color: var(--ink-soft);
  margin-bottom: 0.55rem;
}
input,
select {
  border: 1px solid var(--rule);
  background: #f7fafb;
  padding: 0.35rem 0.45rem;
  color: var(--ink);
  font-family: var(--type-data);
}
.row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.5rem;
}
.presets,
.picker {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin: 0.4rem 0 0.7rem;
}
.presets button,
.picker button,
.link,
.lock {
  border: 1px solid var(--ink);
  background: transparent;
  color: var(--ink);
  padding: 0.25rem 0.5rem;
  cursor: pointer;
  font-size: 0.8rem;
}
.hint {
  font-size: 0.78rem;
  color: var(--ink-soft);
  margin: 0 0 0.5rem;
}
.idv-list {
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  max-height: 16rem;
  overflow: auto;
  border-top: 1px solid var(--grid);
  border-bottom: 1px solid var(--grid);
  padding: 0.3rem 0;
}
.idv {
  display: grid;
  grid-template-columns: auto auto 1fr 4.2rem;
  gap: 0.35rem;
  align-items: center;
  margin: 0;
  flex-direction: row;
}
.idv-n {
  font-family: var(--type-data);
  font-size: 0.75rem;
}
.idv-body {
  font-size: 0.78rem;
  color: var(--ink);
}
.idv-body em {
  display: block;
  font-style: normal;
  color: var(--ink-soft);
  font-size: 0.7rem;
}
.hour {
  width: 4.2rem;
}
.sp {
  display: grid;
  grid-template-columns: 1fr 6.5rem;
  align-items: center;
  gap: 0.4rem;
  margin-bottom: 0.35rem;
  flex-direction: row;
}
.sp span {
  display: flex;
  flex-wrap: wrap;
  gap: 0.25rem 0.4rem;
  align-items: baseline;
  color: var(--ink);
}
.sp small {
  color: var(--ink-soft);
}
.lock.on {
  background: var(--ink);
  color: var(--sheet);
}
.link {
  margin-bottom: 0.8rem;
}
.run {
  width: 100%;
  margin-top: 0.6rem;
  padding: 0.7rem;
  background: var(--ink);
  color: var(--sheet);
  border: 0;
  cursor: pointer;
  font-family: var(--type-display);
  font-size: 1.05rem;
}
.run:disabled {
  opacity: 0.6;
  cursor: wait;
}
.legend {
  display: flex;
  flex-wrap: wrap;
  gap: 0.6rem 1rem;
  padding: 0.55rem 0 0.2rem;
  font-size: 0.82rem;
}
.pen {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 0.35rem;
  margin: 0;
  color: var(--ink);
}
.pen i {
  width: 0.7rem;
  height: 0.7rem;
  display: inline-block;
}
.pen b {
  font-family: var(--type-data);
  font-weight: 600;
}
.picker input {
  flex: 1 1 12rem;
}
.scrub {
  width: 100%;
  margin-top: 0.4rem;
}
.foot {
  margin: 0.5rem 0 0;
  font-size: 0.78rem;
  color: var(--ink-soft);
}
@media (max-width: 980px) {
  .layout {
    grid-template-columns: 1fr;
  }
  .log {
    max-height: none;
  }
  .mast {
    flex-direction: column;
    align-items: start;
  }
}
</style>
