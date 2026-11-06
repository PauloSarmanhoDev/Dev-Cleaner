const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const LABELS = { Language: "Linguagem", Ide: "IDE/Editor", LocalProject: "Projeto Local" };
const $ = (id) => document.getElementById(id);

let targets = [];
const selected = new Set();

function fmt(bytes) {
  const KB = 1024, MB = KB * 1024, GB = MB * 1024;
  if (bytes >= GB) return (bytes / GB).toFixed(2) + " GB";
  if (bytes >= MB) return (bytes / MB).toFixed(1) + " MB";
  if (bytes >= KB) return Math.round(bytes / KB) + " KB";
  return bytes + " B";
}

function setBusy(busy) {
  $("btn-scan").disabled = busy;
  $("btn-clean").disabled = busy || selected.size === 0;
}

function showProgress(frac, text) {
  $("progress").classList.remove("hidden");
  $("bar").style.width = Math.round(frac * 100) + "%";
  $("progress-text").textContent = text;
}

function showWarnings(list) {
  const ul = $("log").querySelector("ul");
  ul.replaceChildren();
  list.slice(0, 50).forEach((w) => {
    const li = document.createElement("li");
    li.textContent = w;
    ul.appendChild(li);
  });
  $("log").classList.toggle("hidden", list.length === 0);
}

function updateSummary() {
  const total = targets.reduce((s, t) => s + t.size, 0);
  const sel = [...selected].reduce((s, i) => s + targets[i].size, 0);
  $("summary").textContent =
    `${targets.length} itens • total ${fmt(total)} • selecionado ${selected.size} (${fmt(sel)})`;
  $("btn-clean").disabled = selected.size === 0;
}

function render() {
  const filter = $("filter").value;
  const tbody = $("table").querySelector("tbody");
  tbody.replaceChildren();
  targets.forEach((t, i) => {
    if (filter && t.category !== filter) return;
    const tr = document.createElement("tr");
    if (t.critical) tr.className = "critical";

    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = selected.has(i);
    cb.onchange = () => { cb.checked ? selected.add(i) : selected.delete(i); updateSummary(); };
    const tdCb = document.createElement("td");
    tdCb.appendChild(cb);

    const badge = document.createElement("span");
    badge.className = "badge " + t.category;
    badge.textContent = LABELS[t.category] ?? t.category;
    const tdType = document.createElement("td");
    tdType.appendChild(badge);

    const tdName = document.createElement("td");
    tdName.className = "name";
    tdName.textContent = (t.critical ? "⚠ " : "") + t.name;

    const tdPath = document.createElement("td");
    tdPath.className = "path";
    tdPath.textContent = t.path;

    const tdFiles = document.createElement("td");
    tdFiles.className = "num";
    tdFiles.textContent = t.files.toLocaleString("pt-BR");
    const tdSize = document.createElement("td");
    tdSize.className = "num";
    tdSize.textContent = fmt(t.size);

    tr.append(tdCb, tdType, tdName, tdPath, tdFiles, tdSize);
    tbody.appendChild(tr);
  });
  const has = targets.length > 0;
  $("table").classList.toggle("hidden", !has);
  $("toolbar").classList.toggle("hidden", !has);
  $("empty").classList.toggle("hidden", has);
  if (!has) $("empty").textContent = "Nada para limpar.";
  updateSummary();
}

async function scan() {
  setBusy(true);
  selected.clear();
  showProgress(0, "Analisando...");
  try {
    const res = await invoke("scan", {
      opts: {
        lang: $("opt-lang").checked,
        ide: $("opt-ide").checked,
        root: $("opt-root").value,
        depth: parseInt($("opt-depth").value, 10) || 6,
      },
    });
    targets = res.targets;
    showWarnings(res.warnings);
    render();
  } catch (e) {
    alert("Erro na análise: " + e);
  } finally {
    $("progress").classList.add("hidden");
    setBusy(false);
  }
}

function openConfirm() {
  const chosen = [...selected].map((i) => targets[i]);
  const total = chosen.reduce((s, t) => s + t.size, 0);
  const critical = chosen.filter((t) => t.critical);
  $("modal-text").textContent = `Remover ${chosen.length} item(ns), liberando ${fmt(total)}? Esta ação é permanente.`;
  $("critical-box").classList.toggle("hidden", critical.length === 0);
  const ul = $("critical-list");
  ul.replaceChildren();
  critical.forEach((t) => {
    const li = document.createElement("li");
    li.textContent = `${t.name} (${t.path})`;
    ul.appendChild(li);
  });
  $("critical-input").value = "";
  $("modal-ok").disabled = critical.length > 0;
  $("modal").classList.remove("hidden");
  $("modal").dataset.critical = critical.length > 0 ? "1" : "";
}

async function doClean() {
  const needsPhrase = $("modal").dataset.critical === "1";
  const confirmation = needsPhrase ? $("critical-input").value : null;
  $("modal").classList.add("hidden");
  setBusy(true);
  showProgress(0, "Removendo...");
  try {
    const res = await invoke("clean", { indices: [...selected], confirmation });
    alert(`Liberado: ${fmt(res.freed)}` + (res.warnings.length ? `\n${res.warnings.length} aviso(s) — veja o log.` : ""));
    await scan();
    if (res.warnings.length) showWarnings(res.warnings);
  } catch (e) {
    alert("Erro na remoção: " + e);
    setBusy(false);
  } finally {
    $("progress").classList.add("hidden");
  }
}

$("btn-scan").onclick = scan;
$("btn-clean").onclick = openConfirm;
$("modal-cancel").onclick = () => $("modal").classList.add("hidden");
$("modal-ok").onclick = doClean;
$("critical-input").oninput = (e) => { $("modal-ok").disabled = e.target.value.trim() !== "APAGAR"; };
$("filter").onchange = render;
$("btn-none").onclick = () => { selected.clear(); render(); };
$("btn-all").onclick = () => {
  const f = $("filter").value;
  targets.forEach((t, i) => { if (!f || t.category === f) selected.add(i); });
  render();
};

listen("scan-progress", (e) => {
  const { done, total, bytes } = e.payload;
  showProgress(total ? done / total : 0, `${done}/${total} • ${fmt(bytes)} encontrados`);
});
listen("clean-progress", (e) => {
  const { done, total } = e.payload;
  showProgress(total ? done / total : 0, `Removendo ${done}/${total}`);
});

