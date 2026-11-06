const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

/* ---------- i18n ---------- */
const I18N = {
  pt: {
    subtitle: "Caches, extensões e artefatos de linguagens, IDEs e projetos",
    opt_lang: "Linguagens/Ferramentas", opt_ide: "IDEs/Editores",
    opt_root: "Pasta de projetos", opt_root_ph: "ex.: E:\\Git (vazio = não procurar)",
    pick: "Escolher...", opt_depth: "Profundidade", opt_older: "Sem uso há",
    opt_perm: "Apagar permanentemente (sem lixeira)",
    scan: "Analisar", use_cache: "Último resultado", cancel: "Cancelar",
    sel_visible: "Marcar visíveis", sel_none: "Desmarcar tudo", remove: "Remover selecionados",
    h_type: "Tipo", h_name: "Nome", h_path: "Caminho", h_files: "Arquivos", h_size: "Tamanho", h_used: "Último uso",
    warnings: "Avisos", confirm_title: "Confirmar remoção", remove_btn: "Remover",
    running_warn: "🔒 Estes programas parecem estar abertos e podem bloquear arquivos:",
    running_continue: "Continuar mesmo assim",
    critical_warn: "⚠ Itens críticos (serão perdidos e precisarão ser reinstalados):",
    type_word: "Digite", all_types: "Todos os tipos",
    Language: "Linguagem", Tool: "Ferramenta", Ide: "IDE/Editor", LocalProject: "Projeto Local",
    start: "Clique em “Analisar” para começar.", nothing: "Nada para limpar.", scanning: "Analisando...",
    removing: "Removendo", found: "encontrados", files: "arquivos",
    summary: (n, t, s, sz) => `${n} itens • total ${t} • selecionado ${s} (${sz})`,
    confirm: (n, sz, perm) => `Remover ${n} item(ns), liberando ${sz}? ` + (perm ? "REMOÇÃO PERMANENTE, sem volta." : "Os itens irão para a lixeira."),
    done: (sz, n, trash) => `Liberado: ${sz} (${n} item(ns))` + (trash ? "\nEnviado para a lixeira." : ""),
    warn_count: (n) => `\n${n} aviso(s): veja o log.`,
    cached: (m) => `Resultado salvo há ${m} min (tamanhos podem estar desatualizados).`,
    cancelled: "Operação cancelada.", err_scan: "Erro na análise: ", err_clean: "Erro na remoção: ",
    audit: (p) => `Registro de auditoria: ${p}`, word: "APAGAR", reveal: "Mostrar na pasta", days: (d) => d + "d",
  },
  en: {
    subtitle: "Caches, extensions and artifacts of languages, IDEs and projects",
    opt_lang: "Languages/Tools", opt_ide: "IDEs/Editors",
    opt_root: "Projects folder", opt_root_ph: "e.g. E:\\Git (empty = don't search)",
    pick: "Browse...", opt_depth: "Depth", opt_older: "Unused for",
    opt_perm: "Delete permanently (skip trash)",
    scan: "Scan", use_cache: "Last result", cancel: "Cancel",
    sel_visible: "Select visible", sel_none: "Clear selection", remove: "Remove selected",
    h_type: "Type", h_name: "Name", h_path: "Path", h_files: "Files", h_size: "Size", h_used: "Last used",
    warnings: "Warnings", confirm_title: "Confirm removal", remove_btn: "Remove",
    running_warn: "🔒 These programs seem to be running and may lock files:",
    running_continue: "Continue anyway",
    critical_warn: "⚠ Critical items (they will be lost and must be reinstalled):",
    type_word: "Type", all_types: "All types",
    Language: "Language", Tool: "Tool", Ide: "IDE/Editor", LocalProject: "Local Project",
    start: "Click “Scan” to start.", nothing: "Nothing to clean.", scanning: "Scanning...",
    removing: "Removing", found: "found", files: "files",
    summary: (n, t, s, sz) => `${n} items • total ${t} • selected ${s} (${sz})`,
    confirm: (n, sz, perm) => `Remove ${n} item(s), freeing ${sz}? ` + (perm ? "PERMANENT deletion, no undo." : "Items will go to the trash."),
    done: (sz, n, trash) => `Freed: ${sz} (${n} item(s))` + (trash ? "\nMoved to the trash." : ""),
    warn_count: (n) => `\n${n} warning(s): see the log.`,
    cached: (m) => `Result saved ${m} min ago (sizes may be outdated).`,
    cancelled: "Operation cancelled.", err_scan: "Scan error: ", err_clean: "Removal error: ",
    audit: (p) => `Audit log: ${p}`, word: "DELETE", reveal: "Show in folder", days: (d) => d + "d",
  },
};
let lang = localStorage.getItem("lang") || ((navigator.language || "en").toLowerCase().startsWith("pt") ? "pt" : "en");
const tr = (k, ...a) => { const v = I18N[lang][k]; return typeof v === "function" ? v(...a) : v ?? k; };

function applyLang() {
  document.documentElement.lang = lang === "pt" ? "pt-BR" : "en";
  document.querySelectorAll("[data-i18n]").forEach((e) => (e.textContent = tr(e.dataset.i18n)));
  document.querySelectorAll("[data-i18n-ph]").forEach((e) => (e.placeholder = tr(e.dataset.i18nPh)));
  const f = $("filter"), cur = f.value;
  f.replaceChildren(new Option(tr("all_types"), ""));
  ["Language", "Tool", "Ide", "LocalProject"].forEach((c) => f.add(new Option(tr(c), c)));
  f.value = cur;
  $("critical-word").textContent = tr("word");
  $("lang").value = lang;
  if (!targets.length) $("empty").textContent = tr("start");
  render();
}

/* ---------- estado ---------- */
let targets = [];
const selected = new Set();
let sortKey = "size", sortDir = -1;
let info = {};

function fmt(bytes) {
  const KB = 1024, MB = KB * 1024, GB = MB * 1024;
  if (bytes >= GB) return (bytes / GB).toFixed(2) + " GB";
  if (bytes >= MB) return (bytes / MB).toFixed(1) + " MB";
  if (bytes >= KB) return Math.round(bytes / KB) + " KB";
  return bytes + " B";
}
const ageDays = (t) => (t.last_used ? Math.floor((Date.now() / 1000 - t.last_used) / 86400) : null);

function setBusy(busy) {
  $("btn-scan").disabled = busy;
  $("btn-cache").disabled = busy;
  $("btn-cancel").disabled = !busy;
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
  list.slice(0, 50).forEach((w) => { const li = document.createElement("li"); li.textContent = w; ul.appendChild(li); });
  $("log").classList.toggle("hidden", list.length === 0);
}

function renderChart() {
  const box = $("chart");
  box.replaceChildren();
  const total = targets.reduce((s, t) => s + t.size, 0);
  if (!total) return box.classList.add("hidden");
  const by = {};
  targets.forEach((t) => (by[t.category] = (by[t.category] || 0) + t.size));
  const stack = document.createElement("div"); stack.className = "stack";
  const legend = document.createElement("div"); legend.className = "legend";
  Object.entries(by).sort((a, b) => b[1] - a[1]).forEach(([c, s]) => {
    const seg = document.createElement("div");
    seg.className = "seg " + c; seg.style.width = (s / total) * 100 + "%"; seg.title = `${tr(c)}: ${fmt(s)}`;
    stack.appendChild(seg);
    const item = document.createElement("span");
    const dot = document.createElement("i"); dot.className = "seg " + c;
    item.append(dot, `${tr(c)}: ${fmt(s)}`);
    legend.appendChild(item);
  });
  box.append(stack, legend);
  box.classList.remove("hidden");
}

function updateSummary() {
  const total = targets.reduce((s, t) => s + t.size, 0);
  const sel = [...selected].reduce((s, i) => s + targets[i].size, 0);
  $("summary").textContent = tr("summary", targets.length, fmt(total), selected.size, fmt(sel));
  $("btn-clean").disabled = selected.size === 0;
}

function visibleOrder() {
  const filter = $("filter").value;
  const idx = targets.map((_, i) => i).filter((i) => !filter || targets[i].category === filter);
  idx.sort((a, b) => {
    const x = targets[a][sortKey], y = targets[b][sortKey];
    const cmp = typeof x === "string" ? x.localeCompare(y) : (x ?? 0) - (y ?? 0);
    return cmp * sortDir;
  });
  return idx;
}

function td(cls, text) { const e = document.createElement("td"); if (cls) e.className = cls; if (text !== undefined) e.textContent = text; return e; }

function render() {
  const tbody = $("table").querySelector("tbody");
  tbody.replaceChildren();
  visibleOrder().forEach((i) => {
    const t = targets[i];
    const tr_ = document.createElement("tr");
    if (t.critical) tr_.className = "critical";

    const cb = document.createElement("input"); cb.type = "checkbox"; cb.checked = selected.has(i);
    cb.onchange = () => { cb.checked ? selected.add(i) : selected.delete(i); updateSummary(); };
    const tdCb = td(); tdCb.appendChild(cb);

    const badge = document.createElement("span"); badge.className = "badge " + t.category; badge.textContent = tr(t.category);
    const tdType = td(); tdType.appendChild(badge);

    const tdName = td("name");
    if (t.running) { const l = document.createElement("span"); l.className = "lock"; l.textContent = "🔒"; l.title = t.running; tdName.appendChild(l); }
    tdName.append((t.critical ? "⚠ " : "") + t.name);

    const d = ageDays(t);
    const reveal = document.createElement("button"); reveal.className = "icon"; reveal.textContent = "📂"; reveal.title = tr("reveal");
    reveal.onclick = () => window.__TAURI__.opener.revealItemInDir(t.path).catch(() => {});
    const tdAct = td(); tdAct.appendChild(reveal);

    tr_.append(tdCb, tdType, tdName, td("path", t.path), td("num", t.files.toLocaleString(lang)), td("num", fmt(t.size)), td("num", d === null ? "-" : tr("days", d)), tdAct);
    tbody.appendChild(tr_);
  });
  const has = targets.length > 0;
  $("table").classList.toggle("hidden", !has);
  $("toolbar").classList.toggle("hidden", !has);
  $("empty").classList.toggle("hidden", has);
  renderChart();
  updateSummary();
}

function applyResult(res) {
  targets = res.targets;
  selected.clear();
  showWarnings(res.warnings);
  $("empty").textContent = res.cancelled ? tr("cancelled") : targets.length ? "" : tr("nothing");
  render();
  if (res.cached_minutes !== null && res.cached_minutes !== undefined) showWarnings([tr("cached", res.cached_minutes)]);
}

/* ---------- ações ---------- */
async function scan() {
  setBusy(true);
  showProgress(0, tr("scanning"));
  try {
    applyResult(await invoke("scan", { opts: {
      lang: $("opt-lang").checked, ide: $("opt-ide").checked,
      root: $("opt-root").value, depth: parseInt($("opt-depth").value, 10) || 6,
      older_than: $("opt-older").value,
    } }));
  } catch (e) { alert(tr("err_scan") + e); }
  finally { $("progress").classList.add("hidden"); setBusy(false); }
}

async function useCache() {
  try { applyResult(await invoke("load_cache")); } catch (e) { alert(e); }
}

function chosenTargets() { return [...selected].map((i) => targets[i]); }

function updateOkButton() {
  const chosen = chosenTargets();
  const needPhrase = chosen.some((t) => t.critical);
  const phraseOk = !needPhrase || ["APAGAR", "DELETE"].includes($("critical-input").value.trim());
  const needRun = chosen.some((t) => t.running);
  $("modal-ok").disabled = !(phraseOk && (!needRun || $("running-ok").checked));
}

function openConfirm() {
  const chosen = chosenTargets();
  const total = chosen.reduce((s, t) => s + t.size, 0);
  const critical = chosen.filter((t) => t.critical);
  const running = chosen.filter((t) => t.running);
  $("modal-text").textContent = tr("confirm", chosen.length, fmt(total), $("opt-perm").checked);
  const fill = (id, list, f) => { const ul = $(id); ul.replaceChildren(); list.forEach((t) => { const li = document.createElement("li"); li.textContent = f(t); ul.appendChild(li); }); };
  fill("critical-list", critical, (t) => `${t.name} (${t.path})`);
  fill("running-list", running, (t) => `${t.running}: ${t.name}`);
  $("critical-box").classList.toggle("hidden", !critical.length);
  $("running-box").classList.toggle("hidden", !running.length);
  $("critical-input").value = ""; $("running-ok").checked = false;
  updateOkButton();
  $("modal").classList.remove("hidden");
}

async function doClean() {
  const chosen = chosenTargets();
  const confirmation = chosen.some((t) => t.critical) ? $("critical-input").value : null;
  const forceRunning = chosen.some((t) => t.running);
  const indices = [...selected];
  $("modal").classList.add("hidden");
  setBusy(true);
  showProgress(0, tr("removing"));
  try {
    const res = await invoke("clean", { indices, confirmation, permanent: $("opt-perm").checked, forceRunning });
    alert(tr("done", fmt(res.freed), res.removed, res.trashed && res.removed > 0) + (res.warnings.length ? tr("warn_count", res.warnings.length) : ""));
    $("progress").classList.add("hidden");
    await scan();
    if (res.warnings.length) showWarnings(res.warnings);
  } catch (e) { alert(tr("err_clean") + e); }
  finally { $("progress").classList.add("hidden"); setBusy(false); }
}

async function pickFolder() {
  const dir = await window.__TAURI__.dialog.open({ directory: true, multiple: false });
  if (dir) $("opt-root").value = dir;
}

/* ---------- eventos ---------- */
$("btn-scan").onclick = scan;
$("btn-cache").onclick = useCache;
$("btn-cancel").onclick = () => invoke("cancel");
$("btn-pick").onclick = pickFolder;
$("btn-clean").onclick = openConfirm;
$("modal-cancel").onclick = () => $("modal").classList.add("hidden");
$("modal-ok").onclick = doClean;
$("critical-input").oninput = updateOkButton;
$("running-ok").onchange = updateOkButton;
$("filter").onchange = render;
$("btn-none").onclick = () => { selected.clear(); render(); };
$("btn-all").onclick = () => { visibleOrder().forEach((i) => selected.add(i)); render(); };
document.querySelectorAll("th[data-sort]").forEach((th) => {
  th.onclick = () => {
    const k = th.dataset.sort;
    if (sortKey === k) sortDir = -sortDir; else { sortKey = k; sortDir = k === "size" || k === "files" ? -1 : 1; }
    render();
  };
});
$("lang").onchange = (e) => { lang = e.target.value; localStorage.setItem("lang", lang); invoke("set_lang", { lang }); applyLang(); };
$("btn-theme").onclick = () => {
  const next = document.documentElement.dataset.theme === "light" ? "dark" : "light";
  document.documentElement.dataset.theme = next; localStorage.setItem("theme", next);
};
document.documentElement.dataset.theme = localStorage.getItem("theme") ||
  (window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark");

listen("scan-progress", (e) => {
  const { done, total, bytes, files } = e.payload;
  showProgress(total ? done / total : 0, `${done}/${total} • ${fmt(bytes)} ${tr("found")} • ${files.toLocaleString(lang)} ${tr("files")}`);
});
listen("clean-progress", (e) => {
  const { done, total } = e.payload;
  showProgress(total ? done / total : 0, `${tr("removing")} ${done}/${total}`);
});

(async () => {
  try {
    info = await invoke("get_info");
    if (!localStorage.getItem("lang")) lang = info.lang;
    $("opt-perm").checked = !!info.permanent_default;
    if (info.older_than_default) $("opt-older").value = info.older_than_default;
    if (info.audit_log) $("audit").textContent = tr("audit", info.audit_log);
  } catch (_) {}
  await invoke("set_lang", { lang });
  applyLang();
  setBusy(false);
  $("btn-cancel").disabled = true;
})();
