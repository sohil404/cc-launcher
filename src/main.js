const { invoke } = window.__TAURI__.core;

const $ = (s) => document.querySelector(s);
const state = { projects: [], config: null, terminals: [], query: "", sel: -1 };

// ── Location color palette (auto-assigned) ──
const LOC_COLORS = [
  { c: "#7aab8e", bg: "rgba(122,171,142,0.12)" },
  { c: "#9a8abf", bg: "rgba(154,138,191,0.12)" },
  { c: "#8aabbf", bg: "rgba(138,171,191,0.12)" },
  { c: "#bf9a8a", bg: "rgba(191,154,138,0.12)" },
  { c: "#b5a87a", bg: "rgba(181,168,122,0.12)" },
  { c: "#8abfab", bg: "rgba(138,191,171,0.12)" },
];
const locMap = {};
function locColor(name) {
  if (!locMap[name]) locMap[name] = LOC_COLORS[Object.keys(locMap).length % LOC_COLORS.length];
  return locMap[name];
}

// ── SVGs ──
const SVG = {
  star: `<svg viewBox="0 0 24 24"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>`,
  sessions: `<svg viewBox="0 0 24 24"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18"/></svg>`,
  clock: `<svg viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>`,
  msgs: `<svg viewBox="0 0 24 24"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>`,
};

// ── Init ──
async function init() {
  show("loading"); hide("project-list"); hide("empty");
  try {
    state.config = await invoke("get_config");
    state.terminals = await invoke("get_available_terminals");
    state.projects = await invoke("get_projects");
    render();
  } catch (e) { console.error("Init:", e); }
  hide("loading");
}

// ── Render ──
function render() {
  const list = $("#project-list");
  const filtered = filter(state.projects, state.query);

  $("#search-count").textContent = filtered.length;

  if (!filtered.length) { hide("project-list"); show("empty"); return; }
  show("project-list"); hide("empty");
  list.innerHTML = "";

  const pinned = filtered.filter((p) => p.pinned);
  const recent = filtered.filter((p) => !p.pinned && (p.last_launched || p.claude.last_active));
  const rest = filtered.filter((p) => !p.pinned && !p.last_launched && !p.claude.last_active);

  let idx = 0;
  if (pinned.length) { list.appendChild(sec("Pinned")); pinned.forEach((p) => list.appendChild(row(p, idx++))); }
  if (recent.length) { list.appendChild(sec("Recent")); recent.forEach((p) => list.appendChild(row(p, idx++))); }
  if (rest.length) { list.appendChild(sec(pinned.length || recent.length ? "All" : "Projects")); rest.forEach((p) => list.appendChild(row(p, idx++))); }

  updateSel();
}

function sec(text) {
  const d = document.createElement("div");
  d.className = "sec";
  d.textContent = text;
  return d;
}

function row(p, idx) {
  const el = document.createElement("div");
  el.className = "row" + (p.pinned ? " pinned" : "");
  el.dataset.idx = idx;
  el.dataset.path = p.path;
  el.style.animationDelay = Math.min(idx * 12, 150) + "ms";

  // Star
  const star = document.createElement("div");
  star.className = "star" + (p.pinned ? " on" : "");
  star.innerHTML = SVG.star;
  star.onclick = (e) => { e.stopPropagation(); pin(p.path); };

  // Body
  const body = document.createElement("div");
  body.className = "row-body";

  const top = document.createElement("div");
  top.className = "row-top";

  const name = document.createElement("span");
  name.className = "row-name";
  name.textContent = p.name;
  top.appendChild(name);

  if (p.location) {
    const loc = document.createElement("span");
    loc.className = "loc";
    const c = locColor(p.location);
    loc.style.color = c.c;
    loc.style.background = c.bg;
    loc.textContent = p.location;
    top.appendChild(loc);
  }

  body.appendChild(top);

  const sub = document.createElement("div");
  sub.className = "row-sub";

  const path = document.createElement("span");
  path.className = "row-path";
  path.textContent = shortPath(p.path);
  sub.appendChild(path);

  body.appendChild(sub);

  // Right side: meta + launch
  const meta = document.createElement("div");
  meta.style.cssText = "display:flex;align-items:center;gap:8px;flex-shrink:0;";

  if (p.claude.session_count > 0) {
    const s = document.createElement("span");
    s.className = "row-meta";
    s.innerHTML = SVG.sessions + " " + p.claude.session_count;
    meta.appendChild(s);
  }
  if (p.claude.message_count > 0) {
    const m = document.createElement("span");
    m.className = "row-meta";
    m.innerHTML = SVG.msgs + " " + fmtCount(p.claude.message_count);
    meta.appendChild(m);
  }
  if (p.claude.last_active_ago) {
    const t = document.createElement("span");
    t.className = "row-meta";
    t.innerHTML = SVG.clock + " " + p.claude.last_active_ago;
    meta.appendChild(t);
  }

  // Action buttons
  const actions = document.createElement("div");
  actions.className = "row-actions";

  if (p.claude.session_count > 0) {
    const res = document.createElement("button");
    res.className = "row-resume";
    res.textContent = "resume";
    res.title = "Pick a session to resume (--resume)";
    res.onclick = (e) => { e.stopPropagation(); launch(p.path, "resume"); };
    actions.appendChild(res);
  }

  const go = document.createElement("button");
  go.className = "row-go";
  go.textContent = "open";
  go.onclick = (e) => { e.stopPropagation(); launch(p.path); };
  actions.appendChild(go);

  el.appendChild(star);
  el.appendChild(body);
  el.appendChild(meta);
  el.appendChild(actions);

  el.onclick = () => { state.sel = idx; updateSel(); };
  el.ondblclick = () => launch(p.path);

  return el;
}

// ── Actions ──
async function launch(path, mode) {
  try { await invoke("launch_project", { path, mode: mode || null }); } catch (e) { console.error("Launch:", e); }
}

async function pin(path) {
  try {
    const isPinned = await invoke("toggle_pin", { path });
    const p = state.projects.find((x) => x.path === path);
    if (p) p.pinned = isPinned;
    sortProjects();
    render();
  } catch (e) { console.error("Pin:", e); }
}

async function refresh() {
  const btn = $("#refresh-btn");
  btn.classList.add("spinning");
  try { state.projects = await invoke("rescan_projects"); render(); } catch (e) { console.error(e); }
  btn.classList.remove("spinning");
}

// ── Selection ──
function updateSel() {
  document.querySelectorAll(".row").forEach((el) => {
    el.classList.toggle("sel", parseInt(el.dataset.idx) === state.sel);
  });
}
function launchSel() {
  const el = document.querySelector(`.row[data-idx="${state.sel}"]`);
  if (el) launch(el.dataset.path);
}

// ── Settings ──
function openSettings() { hide("main-view"); show("settings-view"); renderSettings(); }
function closeSettings() { hide("settings-view"); show("main-view"); }

function renderSettings() {
  // Terminal
  const sel = $("#terminal-select");
  sel.innerHTML = '<option value="auto">Auto-detect</option>';
  const labels = { iterm2:"iTerm2", warp:"Warp", alacritty:"Alacritty", kitty:"Kitty", terminal:"Terminal.app", "gnome-terminal":"GNOME Terminal", konsole:"Konsole", wezterm:"WezTerm", "windows-terminal":"Windows Terminal", cmd:"cmd.exe" };
  for (const t of state.terminals) {
    const o = document.createElement("option"); o.value = t; o.textContent = labels[t]||t; sel.appendChild(o);
  }
  sel.value = state.config.terminal || "auto";
  sel.onchange = () => { state.config.terminal = sel.value; save(); };

  // Flags
  renderFlags();

  // Dirs
  const list = $("#scan-dirs-list");
  list.innerHTML = "";
  for (const dir of state.config.scan_dirs) {
    const d = document.createElement("div"); d.className = "dir-item";
    const t = document.createElement("span"); t.textContent = dir;
    const r = document.createElement("button"); r.className = "rm"; r.textContent = "×";
    r.onclick = () => rmDir(dir);
    d.appendChild(t); d.appendChild(r); list.appendChild(d);
  }

  // Depth
  const depth = $("#scan-depth"), dv = $("#depth-value");
  depth.value = state.config.scan_depth; dv.textContent = state.config.scan_depth;
  depth.oninput = () => { dv.textContent = depth.value; };
  depth.onchange = () => { state.config.scan_depth = parseInt(depth.value); save(); };
}

// ── Flag Multi-Select ──
const TOGGLES = [
  { flag: "--dangerously-skip-permissions", label: "skip perms", warn: true },
  { flag: "--bare", label: "bare" },
  { flag: "--ide", label: "IDE" },
  { flag: "--continue", label: "continue" },
];
const DROPS = [
  { key: "permission-mode", label: "Mode", opts: [["","Default"],["auto","Auto"],["plan","Plan"],["acceptEdits","Accept Edits"],["bypassPermissions","Bypass"]] },
  { key: "model", label: "Model", opts: [["","Default"],["sonnet","Sonnet"],["opus","Opus"],["haiku","Haiku"]] },
  { key: "effort", label: "Effort", opts: [["","Default"],["low","Low"],["medium","Med"],["high","High"],["max","Max"]] },
];

function parseFlags(s) {
  const tokens = (s||"").split(/\s+/).filter(Boolean);
  const active = new Set(), drops = {}, extra = [];
  const knownT = TOGGLES.map(t=>t.flag), knownD = DROPS.map(d=>"--"+d.key);
  for (let i=0;i<tokens.length;i++) {
    if (knownT.includes(tokens[i])) active.add(tokens[i]);
    else if (knownD.includes(tokens[i])&&i+1<tokens.length) { drops[tokens[i].slice(2)]=tokens[++i]; }
    else extra.push(tokens[i]);
  }
  return { active, drops, extra: extra.join(" ") };
}
function composeFlags(a,d,e) {
  const p=[]; a.forEach(f=>p.push(f));
  for (const[k,v] of Object.entries(d)) if(v) p.push("--"+k,v);
  if(e.trim()) p.push(e.trim());
  return p.join(" ");
}

function renderFlags() {
  const parsed = parseFlags(state.config.launch_flags);

  const chips = $("#flag-chips"); chips.innerHTML = "";
  for (const t of TOGGLES) {
    const c = document.createElement("button");
    c.className = "chip" + (parsed.active.has(t.flag)?" on":"") + (t.warn?" warn":"");
    c.textContent = t.label;
    c.onclick = () => {
      if(parsed.active.has(t.flag)) parsed.active.delete(t.flag); else parsed.active.add(t.flag);
      state.config.launch_flags = composeFlags(parsed.active,parsed.drops,parsed.extra);
      save(); renderFlags();
    };
    chips.appendChild(c);
  }

  const drops = $("#flag-dropdowns"); drops.innerHTML = "";
  for (const d of DROPS) {
    const g = document.createElement("div"); g.className = "drop-group";
    const l = document.createElement("label"); l.textContent = d.label;
    const s = document.createElement("select");
    for (const[v,t] of d.opts) { const o=document.createElement("option"); o.value=v; o.textContent=t; s.appendChild(o); }
    s.value = parsed.drops[d.key]||"";
    s.onchange = () => { parsed.drops[d.key]=s.value; state.config.launch_flags=composeFlags(parsed.active,parsed.drops,parsed.extra); save(); renderFlags(); };
    g.appendChild(l); g.appendChild(s); drops.appendChild(g);
  }

  const extra = $("#extra-flags-input");
  extra.value = parsed.extra;
  extra.onchange = () => { parsed.extra=extra.value; state.config.launch_flags=composeFlags(parsed.active,parsed.drops,parsed.extra); save(); renderFlags(); };

  $("#computed-flags").textContent = "$ claude " + (state.config.launch_flags || "(default)");
}

async function addDir() {
  const input = $("#new-dir-input"), dir = input.value.trim();
  if (!dir||state.config.scan_dirs.includes(dir)) { input.value=""; return; }
  state.config.scan_dirs.push(dir); input.value="";
  await save(); renderSettings(); refresh();
}
async function rmDir(dir) {
  state.config.scan_dirs = state.config.scan_dirs.filter(d=>d!==dir);
  await save(); renderSettings(); refresh();
}
async function save() {
  try { await invoke("update_config",{configUpdate:state.config}); } catch(e) { console.error(e); }
}

// ── Helpers ──
function filter(projects, q) {
  if (!q) return projects;
  const lq = q.toLowerCase();
  return projects.filter(p => p.name.toLowerCase().includes(lq) || p.path.toLowerCase().includes(lq));
}
function sortProjects() {
  state.projects.sort((a,b) => {
    if (a.pinned!==b.pinned) return b.pinned?1:-1;
    const at=a.claude?.last_active||a.last_launched||"", bt=b.claude?.last_active||b.last_launched||"";
    if(bt!==at) return bt.localeCompare(at);
    return a.name.toLowerCase().localeCompare(b.name.toLowerCase());
  });
}
function shortPath(p) {
  const home = p.match(/^\/Users\/[^/]+/)?.[0];
  if (home) p = p.replace(home,"~");
  p = p.replace(/^\/Volumes\/([^/]+)\//,"$1:/");
  return p;
}
function fmtCount(n) { return n>=1000?(n/1000).toFixed(1).replace(/\.0$/,"")+"k":n.toString(); }
function show(id) { document.getElementById(id)?.classList.remove("hidden"); }
function hide(id) { document.getElementById(id)?.classList.add("hidden"); }

// ── Events ──
document.addEventListener("DOMContentLoaded", init);

$("#search").addEventListener("input", e => { state.query=e.target.value; state.sel=-1; render(); });
$("#refresh-btn").addEventListener("click", refresh);
$("#settings-btn").addEventListener("click", openSettings);
$("#settings-back").addEventListener("click", closeSettings);
$("#add-dir-btn").addEventListener("click", addDir);
$("#new-dir-input").addEventListener("keydown", e => { if(e.key==="Enter") addDir(); });

document.addEventListener("keydown", e => {
  if ((e.metaKey||e.ctrlKey)&&e.key==="k") { e.preventDefault(); $("#search").focus(); $("#search").select(); return; }
  if ((e.metaKey||e.ctrlKey)&&e.key===",") { e.preventDefault(); openSettings(); return; }
  if (e.key==="Escape") {
    if (!$("#settings-view").classList.contains("hidden")) closeSettings();
    else if (state.query) { state.query=""; $("#search").value=""; state.sel=-1; render(); }
    return;
  }
  const rows = document.querySelectorAll(".row");
  if (!rows.length) return;
  if (e.key==="ArrowDown") { e.preventDefault(); state.sel=Math.min(state.sel+1,rows.length-1); updateSel(); rows[state.sel]?.scrollIntoView({block:"nearest"}); }
  if (e.key==="ArrowUp") { e.preventDefault(); state.sel=Math.max(state.sel-1,0); updateSel(); rows[state.sel]?.scrollIntoView({block:"nearest"}); }
  if (e.key==="Enter"&&state.sel>=0) { e.preventDefault(); launchSel(); }
});
