/** Operating window (Downs & Vogel 1993) and interlock trips (TEFUNC / ISD). */

export const LIMITS = {
  7: {
    scaleLo: 2500,
    scaleHi: 3000,
    hiOp: 2895,
    hiShutdown: 3000,
  },
  8: {
    scaleLo: 0,
    scaleHi: 100,
    loOp: 50,
    hiOp: 100,
  },
  9: {
    scaleLo: 100,
    scaleHi: 180,
    hiOp: 150,
    hiShutdown: 175,
  },
  12: {
    scaleLo: 0,
    scaleHi: 100,
    loOp: 30,
    hiOp: 70,
  },
  13: {
    scaleLo: 2400,
    scaleHi: 3000,
    hiOp: 2895,
  },
  15: {
    scaleLo: 0,
    scaleHi: 100,
    loOp: 30,
    hiOp: 70,
  },
  40: { scaleLo: 0, scaleHi: 100 },
  41: { scaleLo: 0, scaleHi: 100 },
};

export const KPI_STRIP = [7, 9, 8, 12, 15, 40, 41];

export function exceptionStatus(n, value) {
  const lim = LIMITS[n];
  if (value == null || Number.isNaN(value) || !lim) return "normal";
  if (lim.hiShutdown != null && value >= lim.hiShutdown) return "trip";
  if (lim.loShutdown != null && value <= lim.loShutdown) return "trip";
  if (lim.hiOp != null && value > lim.hiOp) return "advisory";
  if (lim.loOp != null && value < lim.loOp) return "advisory";
  return "normal";
}

export function formatPv(v) {
  if (v == null || Number.isNaN(Number(v))) return "—";
  const n = Number(v);
  const a = Math.abs(n);
  if (a >= 1000) return n.toFixed(0);
  if (a >= 100) return n.toFixed(1);
  if (a >= 10) return n.toFixed(2);
  return n.toFixed(3);
}

export function fracOnScale(value, lo, hi) {
  if (value == null || Number.isNaN(value) || hi === lo) return 0;
  return Math.min(1, Math.max(0, (value - lo) / (hi - lo)));
}
