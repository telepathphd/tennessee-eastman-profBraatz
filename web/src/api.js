export async function loadCatalog() {
  const r = await fetch("/api/catalog");
  if (!r.ok) throw new Error(`catalog ${r.status}`);
  return r.json();
}

export async function runSim(body) {
  const r = await fetch("/api/simulate", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const data = await r.json();
  if (!r.ok) throw new Error(data.error || `simulate ${r.status}`);
  return data;
}

/** Download mimo-sim CSV (time, MV*, CV*) from recorded channels. */
export async function exportMimoCsv({ time_s, mv, cv, record_every = 60 }) {
  const r = await fetch("/api/export/mimo-csv", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ time_s, mv, cv, record_every }),
  });
  if (!r.ok) {
    const data = await r.json().catch(() => ({}));
    throw new Error(data.error || `export ${r.status}`);
  }
  return r.text();
}
