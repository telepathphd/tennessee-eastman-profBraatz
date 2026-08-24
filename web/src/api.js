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
