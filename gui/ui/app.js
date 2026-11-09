const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const $ = (id) => document.getElementById(id);

/* ---------- i18n ---------- */
const I18N = {
  pt: {
    subtitle: "Limpeza profunda de disco, caches e artefatos de build para desenvolvedores",
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
    ok: "OK", close: "Fechar", error: "Erro", done_title: "Concluído", save: "Salvar",
    supported: "Suportados", supported_title: "Linguagens, IDEs e ferramentas suportadas",
    supported_count: (n) => `${n} entradas`, h_what: "O que é limpo",
    settings: "Configurações", settings_title: "Configurações",
    cfg_lang: "Idioma padrão", cfg_lang_auto: "Automático (sistema)",
    cfg_older: "Sem uso há (padrão)", cfg_perm: "Apagar permanentemente por padrão",
    cfg_exclude: "Caminhos excluídos (um por linha)", cfg_extra: "Alvos extras",
    cfg_add: "Adicionar alvo", cfg_name: "Nome", cfg_path: "Caminho", cfg_category: "Categoria",
    cfg_critical: "Crítico", cfg_remove: "Remover", cfg_note: "Salvar reescreve o arquivo (comentários feitos à mão são perdidos).",
    cfg_saved: (p) => `Configuração salva em ${p}`, cfg_load_warn: (w) => `Aviso ao ler a configuração: ${w}`,
    cat_tool: "Ferramenta", cat_language: "Linguagem", cat_ide: "IDE/Editor", cat_local: "Projeto Local",
    critical_tip: "Item crítico: exige confirmação reforçada (APAGAR) porque pode conter dados ou levar tempo para baixar/reinstalar.",
    info_btn: "Sobre este item e consequências ao apagar",
  },
  en: {
    subtitle: "Deep disk & build-artifact cleaner for developers",
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
    ok: "OK", close: "Close", error: "Error", done_title: "Done", save: "Save",
    supported: "Supported", supported_title: "Supported languages, IDEs and tools",
    supported_count: (n) => `${n} entries`, h_what: "What is cleaned",
    settings: "Settings", settings_title: "Settings",
    cfg_lang: "Default language", cfg_lang_auto: "Automatic (system)",
    cfg_older: "Unused for (default)", cfg_perm: "Delete permanently by default",
    cfg_exclude: "Excluded paths (one per line)", cfg_extra: "Extra targets",
    cfg_add: "Add target", cfg_name: "Name", cfg_path: "Path", cfg_category: "Category",
    cfg_critical: "Critical", cfg_remove: "Remove", cfg_note: "Saving rewrites the file (hand-written comments are lost).",
    cfg_saved: (p) => `Configuration saved to ${p}`, cfg_load_warn: (w) => `Warning while reading the configuration: ${w}`,
    cat_tool: "Tool", cat_language: "Language", cat_ide: "IDE/Editor", cat_local: "Local Project",
    critical_tip: "Critical item: requires extra confirmation (DELETE) because it may contain data or take time to re-download/reinstall.",
    info_btn: "About this item and consequences of deleting it",
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
let names = []; // nomes dos alvos no idioma atual (mesmos índices de `targets`)
let infos = []; // explicações dos alvos no idioma atual (mesmos índices de `targets`)
const nameOf = (i) => names[i] ?? targets[i].name;
const infoOf = (i) => infos[i] ?? "";

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
    const x = sortKey === "name" ? nameOf(a) : targets[a][sortKey], y = sortKey === "name" ? nameOf(b) : targets[b][sortKey];
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
    if (t.critical) { tdName.title = tr("critical_tip"); }
    tdName.append((t.critical ? "⚠ " : "") + nameOf(i));

    const d = ageDays(t);
    const tdAct = td("act");
    const infoBtn = document.createElement("button"); infoBtn.className = "icon info-btn"; infoBtn.textContent = "i"; infoBtn.title = tr("info_btn");
    infoBtn.onclick = () => showMessage(infoOf(i), nameOf(i));
    tdAct.appendChild(infoBtn);
    if (!t.kind || t.kind === "Dir") { // Docker não tem pasta para mostrar
      const reveal = document.createElement("button"); reveal.className = "icon"; reveal.textContent = "📂"; reveal.title = tr("reveal");
      reveal.onclick = () => window.__TAURI__.opener.revealItemInDir(t.path).catch(() => {});
      tdAct.appendChild(reveal);
    }

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

async function refreshNames() {
  try {
    [names, infos] = await Promise.all([invoke("target_names"), invoke("target_infos")]);
  } catch (_) { names = []; infos = []; }
}

/* Substitui o alert() nativo do navegador por um diálogo da própria interface. */
function showMessage(text, title) {
  return new Promise((resolve) => {
    $("msg-title").textContent = title || "";
    $("msg-text").textContent = text;
    $("msg-modal").classList.remove("hidden");
    const close = () => { $("msg-modal").classList.add("hidden"); document.removeEventListener("keydown", onKey); resolve(); };
    const onKey = (e) => { if (e.key === "Escape" || e.key === "Enter") close(); };
    document.addEventListener("keydown", onKey);
    $("msg-ok").onclick = close;
    $("msg-ok").focus();
  });
}

async function applyResult(res) {
  targets = res.targets;
  await refreshNames();
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
    await applyResult(await invoke("scan", { opts: {
      lang: $("opt-lang").checked, ide: $("opt-ide").checked,
      root: $("opt-root").value, depth: parseInt($("opt-depth").value, 10) || 6,
      older_than: $("opt-older").value,
    } }));
  } catch (e) { await showMessage(tr("err_scan") + e, tr("error")); }
  finally { $("progress").classList.add("hidden"); setBusy(false); }
}

async function useCache() {
  try { await applyResult(await invoke("load_cache")); } catch (e) { await showMessage(String(e), tr("error")); }
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
    await showMessage(tr("done", fmt(res.freed), res.removed, res.trashed && res.removed > 0) + (res.warnings.length ? tr("warn_count", res.warnings.length) : ""), tr("done_title"));
    $("progress").classList.add("hidden");
    await scan();
    if (res.warnings.length) showWarnings(res.warnings);
  } catch (e) { await showMessage(tr("err_clean") + e, tr("error")); }
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
$("lang").onchange = async (e) => {
  lang = e.target.value; localStorage.setItem("lang", lang);
  await invoke("set_lang", { lang });
  await refreshNames();
  applyLang();
  supported = null;
  if (!$("supported-modal").classList.contains("hidden")) renderSupported(true);
  if (!$("settings-modal").classList.contains("hidden")) applySettingsLang();
};
/* ---------- lista de suportados ---------- */
let supported = null;
async function renderSupported(forceReload = false) {
  if (!supported || forceReload) supported = await invoke("get_supported");
  $("supported-count").textContent = tr("supported_count", supported.length);
  const tb = $("supported-table").querySelector("tbody");
  tb.replaceChildren();
  supported.forEach((s) => {
    const badge = document.createElement("span"); badge.className = "badge " + s.category; badge.textContent = tr(s.category);
    const tdType = td(); tdType.appendChild(badge);
    const tdAct = td("act");
    const infoBtn = document.createElement("button"); infoBtn.className = "icon info-btn"; infoBtn.textContent = "i"; infoBtn.title = tr("info_btn");
    infoBtn.onclick = () => showMessage(s.info, s.name);
    tdAct.appendChild(infoBtn);
    const tr_ = document.createElement("tr");
    tr_.append(tdType, td("name", s.name), td("path", s.detail), tdAct);
    tb.appendChild(tr_);
  });
}
$("btn-supported").onclick = async () => {
  try { await renderSupported(); $("supported-modal").classList.remove("hidden"); }
  catch (e) { await showMessage(String(e), tr("error")); }
};
$("supported-close").onclick = () => $("supported-modal").classList.add("hidden");

/* ---------- configurações (config.toml) ---------- */
const CATEGORIES = ["tool", "language", "ide", "local"];
function applySettingsLang() {
  document.querySelectorAll("#settings-modal [data-i18n]").forEach((e) => (e.textContent = tr(e.dataset.i18n)));
  const sel = $("cfg-lang"), cur = sel.value;
  sel.replaceChildren(new Option(tr("cfg_lang_auto"), ""), new Option("Português", "pt"), new Option("English", "en"));
  sel.value = cur;
  document.querySelectorAll("#extras-box select.x-cat").forEach((s) => {
    const c = s.value;
    s.replaceChildren(...CATEGORIES.map((k) => new Option(tr("cat_" + k), k)));
    s.value = c;
  });
  document.querySelectorAll("#extras-box [data-ph]").forEach((e) => (e.placeholder = tr(e.dataset.ph)));
  document.querySelectorAll("#extras-box .x-del").forEach((b) => (b.textContent = tr("cfg_remove")));
  document.querySelectorAll("#extras-box .x-crit-l").forEach((b) => (b.lastChild.textContent = " " + tr("cfg_critical")));
}
function addExtraRow(x = {}) {
  const row = document.createElement("div"); row.className = "extra-row";
  const name = document.createElement("input"); name.type = "text"; name.className = "x-name"; name.dataset.ph = "cfg_name"; name.value = x.name || "";
  const path = document.createElement("input"); path.type = "text"; path.className = "x-path"; path.dataset.ph = "cfg_path"; path.value = x.path || "";
  const pick = document.createElement("button"); pick.textContent = "…";
  pick.onclick = async () => { const d = await window.__TAURI__.dialog.open({ directory: true, multiple: false }); if (d) path.value = d; };
  const cat = document.createElement("select"); cat.className = "x-cat";
  CATEGORIES.forEach((k) => cat.add(new Option(k, k)));
  cat.value = CATEGORIES.includes(x.category) ? x.category : "tool";
  const critL = document.createElement("label"); critL.className = "x-crit-l";
  const crit = document.createElement("input"); crit.type = "checkbox"; crit.className = "x-crit"; crit.checked = !!x.critical;
  critL.append(crit, document.createTextNode(""));
  const del = document.createElement("button"); del.className = "x-del"; del.onclick = () => row.remove();
  row.append(name, path, pick, cat, critL, del);
  $("extras-box").appendChild(row);
}
$("btn-settings").onclick = async () => {
  try {
    const v = await invoke("get_config");
    const c = v.config;
    $("cfg-lang").value = c.language || "";
    $("cfg-older").value = c.older_than || "";
    $("cfg-perm").checked = !!c.permanent;
    $("cfg-exclude").value = (c.exclude || []).join("\n");
    $("extras-box").replaceChildren();
    (c.extra || []).forEach((x) => addExtraRow({ ...x, category: (x.category || "tool").toLowerCase() }));
    $("cfg-path").textContent = v.path || "";
    $("cfg-warn").textContent = v.warning ? tr("cfg_load_warn", v.warning) : "";
    $("cfg-error").textContent = "";
    applySettingsLang();
    $("settings-modal").classList.remove("hidden");
  } catch (e) { await showMessage(String(e), tr("error")); }
};
$("cfg-add").onclick = () => { addExtraRow(); applySettingsLang(); };
$("cfg-cancel").onclick = () => $("settings-modal").classList.add("hidden");
$("cfg-save").onclick = async () => {
  const config = {
    language: $("cfg-lang").value || null,
    older_than: $("cfg-older").value || null,
    permanent: $("cfg-perm").checked,
    exclude: $("cfg-exclude").value.split("\n"),
    extra: [...document.querySelectorAll("#extras-box .extra-row")].map((r) => ({
      name: r.querySelector(".x-name").value, path: r.querySelector(".x-path").value,
      category: r.querySelector(".x-cat").value, critical: r.querySelector(".x-crit").checked,
    })),
  };
  try {
    const p = await invoke("save_config", { config });
    $("settings-modal").classList.add("hidden");
    $("opt-perm").checked = config.permanent;
    if (config.older_than) $("opt-older").value = config.older_than;
    if (config.language && config.language !== lang) { $("lang").value = config.language; $("lang").onchange({ target: $("lang") }); }
    await showMessage(tr("cfg_saved", p), tr("settings_title"));
  } catch (e) { $("cfg-error").textContent = String(e); }
};

$("btn-theme").onclick = () => {
  const next = document.documentElement.dataset.theme === "light" ? "dark" : "light";
  document.documentElement.dataset.theme = next; localStorage.setItem("theme", next);
};
document.documentElement.dataset.theme = localStorage.getItem("theme") ||
  (window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark");

listen("scan-progress", (e) => {
  const { done, total, bytes, files, current } = e.payload;
  showProgress(total ? done / total : 0, `${done}/${total} • ${fmt(bytes)} ${tr("found")} • ${files.toLocaleString(lang)} ${tr("files")}` + (current ? ` • ${current}` : ""));
  $("progress-text").title = current || "";
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
