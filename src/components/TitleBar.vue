<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch, nextTick } from "vue"
import { X, Sun, Moon, Monitor, Minus, Square, Copy, Settings, Folder, Eye, Code } from "@lucide/vue"
import appIcon from "../../src-tauri/icons/32x32.png"
import { activeTabId, tabs, editorMode } from "@/lib/file-store"

interface MenuItem {
  id?: string
  label?: string
  shortcut?: string
  type?: "item" | "separator"
}

interface Menu {
  id: string
  label: string
  items: MenuItem[]
}

const emit = defineEmits<{
  (e: "menuAction", id: string): void
  (e: "tabSelect", id: number): void
  (e: "tabClose", id: number): void
}>()

// Just the File menu — the titlebar stays minimal. The gear button on
// the right opens a settings/info modal for everything else.
const menus: Menu[] = [
  {
    id: "file",
    label: "文件",
    items: [
      { id: "new", label: "新建", shortcut: "Ctrl+N" },
      { id: "open", label: "打开…", shortcut: "Ctrl+O" },
      { id: "save", label: "保存", shortcut: "Ctrl+S" },
      { type: "separator" },
      { id: "export-html", label: "导出为 HTML" },
      { id: "export-word", label: "导出为 Word" },
      { id: "export-pdf", label: "导出为 PDF" },
      { id: "export-png", label: "导出为 PNG" },
      { type: "separator" },
      { id: "set-default", label: "设为默认 Markdown 应用" },
    ],
  },
]

const openMenu = ref<string | null>(null)
const showSettings = ref(false)

function openSettings() {
  showSettings.value = true
}
function closeSettings() {
  showSettings.value = false
}
function onKeydown(e: KeyboardEvent) {
  if (e.key === "Escape" && showSettings.value) closeSettings()
}

// Display settings — single source of truth lives in this component
// (persisted in localStorage and applied to <html> + the editor CSS var).
// Default font size is index 1 (中, 15.5px) to match the CSS var defined
// in style.css. Using `|| 1` would clobber a stored "0" (小), so we
// distinguish missing from zero explicitly.
const FONT_SIZE_DEFAULT = 1
const storedFontSize = localStorage.getItem("fontSize")
const fontSize = ref(storedFontSize !== null ? Number(storedFontSize) : FONT_SIZE_DEFAULT)
const theme = ref<"light" | "dark" | "auto">(
  (localStorage.getItem("theme") as "light" | "dark" | "auto") || "auto",
)

const FONT_STEPS = [
  { label: "小", px: "14px" },
  { label: "中", px: "15.5px" },
  { label: "大", px: "17px" },
  { label: "特大", px: "19px" },
]

function applyDisplay() {
  const root = document.documentElement
  const step = FONT_STEPS[fontSize.value] ?? FONT_STEPS[1]
  root.style.setProperty("--editor-font-size", step.px)
  if (theme.value === "auto") {
    const dark = window.matchMedia("(prefers-color-scheme: dark)").matches
    root.classList.toggle("dark", dark)
  } else {
    root.classList.toggle("dark", theme.value === "dark")
  }
  localStorage.setItem("fontSize", String(fontSize.value))
  localStorage.setItem("theme", theme.value)
}

function bumpFont(delta: number) {
  fontSize.value = Math.max(0, Math.min(FONT_STEPS.length - 1, fontSize.value + delta))
  applyDisplay()
}

function cycleTheme() {
  theme.value = theme.value === "light" ? "dark" : theme.value === "dark" ? "auto" : "light"
  applyDisplay()
}

function toggleEditorMode() {
  editorMode.value = editorMode.value === "wysiwyg" ? "source" : "wysiwyg"
}

const currentFontLabel = () => FONT_STEPS[fontSize.value]?.label ?? "中"
const currentThemeLabel = () =>
  theme.value === "auto" ? "跟随系统" : theme.value === "dark" ? "深色" : "浅色"

function onDocumentMousedown(event: MouseEvent) {
  const target = event.target as HTMLElement | null
  if (!target?.closest(".titlebar-menu")) openMenu.value = null
}

function onMenuFocusOut(event: FocusEvent) {
  const next = event.relatedTarget as HTMLElement | null
  if (!next?.closest(".titlebar-menu")) openMenu.value = null
}

function onWindowBlur() {
  openMenu.value = null
}

type WindowApi = Awaited<
  ReturnType<typeof import("@tauri-apps/api/window")["getCurrentWindow"]>
>

let appWindow: WindowApi | null = null
let unlistenResized: (() => void) | undefined
const isMaximized = ref(false)

async function ensureWindow() {
  if (!appWindow) {
    const { getCurrentWindow } = await import("@tauri-apps/api/window")
    appWindow = getCurrentWindow()
  }
  return appWindow
}

async function syncMaximized() {
  try {
    isMaximized.value = await (await ensureWindow()).isMaximized()
  } catch {
    isMaximized.value = false
  }
}

async function minimize() {
  try { await (await ensureWindow()).minimize() } catch { /* not Tauri */ }
}
async function toggleMaximize() {
  try {
    await (await ensureWindow()).toggleMaximize()
    await syncMaximized()
  } catch { /* not Tauri */ }
}
async function closeWindow() {
  try { await (await ensureWindow()).close() } catch { /* not Tauri */ }
}

let mql: MediaQueryList | null = null
function onSystemThemeChange() {
  if (theme.value === "auto") applyDisplay()
}

onMounted(async () => {
  applyDisplay()
  document.addEventListener("mousedown", onDocumentMousedown)
  window.addEventListener("blur", onWindowBlur)
  await syncMaximized()
  try {
    unlistenResized = await (await ensureWindow()).onResized(() => syncMaximized())
  } catch { /* not Tauri */ }
  mql = window.matchMedia("(prefers-color-scheme: dark)")
  mql.addEventListener("change", onSystemThemeChange)
})

onUnmounted(() => {
  document.removeEventListener("mousedown", onDocumentMousedown)
  window.removeEventListener("blur", onWindowBlur)
  unlistenResized?.()
  mql?.removeEventListener("change", onSystemThemeChange)
})

// Tab track — a recessed slot (凹槽) that holds each tab as an
// elliptical pill. A separate "slider" indicator div slides along the
// track to highlight the active tab. The track's `p-1` (4px) padding
// is the same as the indicator's `left-1` + `top-1` initial offset, so
// `offsetLeft - 4` of a pill gives the indicator's translateX directly.
const tabRefs = ref<HTMLElement[]>([])
const indicatorX = ref(0)
const indicatorW = ref(0)

function setTabRef(el: unknown, idx: number) {
  if (el instanceof HTMLElement) tabRefs.value[idx] = el
}

function updateIndicator() {
  const idx = tabs.value.findIndex(t => t.id === activeTabId.value)
  const el = tabRefs.value[idx]
  if (!el) return
  indicatorX.value = el.offsetLeft - 4
  indicatorW.value = el.offsetWidth
}

watch(activeTabId, () => nextTick(updateIndicator))
watch(() => tabs.value.length, () => nextTick(updateIndicator))
// Window resize can change tab widths when the titlebar narrows —
// since tabs shrink to fit, the indicator has to follow the new size.
function onWindowResize() {
  nextTick(updateIndicator)
}
onMounted(() => {
  nextTick(updateIndicator)
  window.addEventListener("resize", onWindowResize)
  document.addEventListener("keydown", onKeydown)
  // Test hook: ?openSettings=1 opens the settings modal for screenshots.
  if (new URLSearchParams(location.search).get("openSettings") === "1") {
    showSettings.value = true
  }
})
onUnmounted(() => {
  window.removeEventListener("resize", onWindowResize)
  document.removeEventListener("keydown", onKeydown)
})
</script>

<template>
  <div
    data-tauri-drag-region="deep"
    class="titlebar flex h-10 shrink-0 select-none items-center border-b border-[var(--line)] bg-[var(--surface)] pl-3 pr-1 text-[var(--ink)]"
  >
    <!-- LEFT: app icon + menus -->
    <div
      class="titlebar-menu flex shrink-0 items-center gap-0.5 border-r border-[var(--line)] pr-2.5"
      data-tauri-no-drag
      @focusout="onMenuFocusOut"
    >
      <img :src="appIcon" class="mr-1.5 size-4" alt="" />
      <div v-for="menu in menus" :key="menu.id" class="relative">
        <button
          type="button"
          class="flex h-7 items-center gap-1.5 rounded-md px-2 text-[13px] text-[var(--ink)] transition-colors hover:bg-black/5 dark:hover:bg-white/10"
          :class="openMenu === menu.id ? 'bg-black/5 dark:bg-white/10' : ''"
          :title="menu.label"
          @click="openMenu = openMenu === menu.id ? null : menu.id"
        >
          <Folder v-if="menu.id === 'file'" class="size-4" />
          <span v-else>{{ menu.label }}</span>
        </button>
        <div
          v-if="openMenu === menu.id"
          class="absolute top-full left-0 z-50 mt-1 w-60 rounded-md border border-[var(--line)] bg-[var(--surface)] py-1 text-[var(--ink)] shadow-lg"
        >
          <template v-for="item in menu.items" :key="item.id ?? item.type">
            <div v-if="item.type === 'separator'" class="my-1 h-px bg-[var(--line)]" />
            <button
              v-else
              type="button"
              class="flex w-full items-center justify-between px-3 py-1.5 text-left text-[13px] hover:bg-black/5 dark:hover:bg-white/10"
              @click="item.id && emit('menuAction', item.id); openMenu = null"
            >
              <span>{{ item.label }}</span>
              <span v-if="item.shortcut" class="text-xs text-[var(--muted)]">
                {{ item.shortcut }}
              </span>
            </button>
          </template>
        </div>
      </div>
      <!-- view-mode toggle: icon shows the action it will perform.
           In preview (default) we show `Code` ("click to see source");
           in source mode we show `Eye` ("click to go back to rendered"). -->
      <button
        type="button"
        class="flex h-7 w-7 items-center justify-center rounded-md transition-colors"
        :class="
          editorMode === 'source'
            ? 'bg-[var(--accent-soft)] text-[var(--accent)]'
            : 'text-[var(--muted)] hover:bg-black/5 hover:text-[var(--ink)] dark:hover:bg-white/10'
        "
        :title="editorMode === 'wysiwyg' ? '查看 Markdown 源码' : '返回预览模式'"
        @click="toggleEditorMode"
      >
        <Code v-if="editorMode === 'wysiwyg'" class="size-4" />
        <Eye v-else class="size-4" />
      </button>
    </div>

    <!-- MIDDLE: tab track — fills the entire middle area as a recessed
         slot. Tabs (elliptical pills) live inside, anchored to the LEFT
         edge of the track so they grow left → right. The sliding
         indicator rides the active pill. -->
    <div
      class="titlebar-tabs flex h-full min-w-0 flex-1 items-center overflow-hidden px-3"
    >
      <div
        class="tab-track relative flex w-full items-center gap-0.5 rounded-full bg-black/[0.05] p-1 shadow-[inset_0_1px_3px_rgba(0,0,0,0.08),inset_0_-1px_0_rgba(0,0,0,0.02)] dark:bg-white/[0.06] dark:shadow-[inset_0_1px_3px_rgba(0,0,0,0.4),inset_0_-1px_0_rgba(255,255,255,0.04)]"
      >
        <!-- sliding indicator (the "slider knob" inside the slot) -->
        <div
          class="tab-indicator pointer-events-none absolute top-1 left-1 h-[calc(100%-8px)] rounded-full bg-[var(--elevated)] shadow-[0_1px_2px_rgba(0,0,0,0.12),0_0_0_0.5px_rgba(0,0,0,0.06)] transition-all duration-200 ease-out dark:shadow-[0_1px_3px_rgba(0,0,0,0.5),0_0_0_0.5px_rgba(255,255,255,0.08)]"
          :style="{ transform: `translateX(${indicatorX}px)`, width: `${indicatorW}px` }"
        ></div>
        <button
          v-for="(tab, i) in tabs"
          :key="tab.id"
          :ref="(el) => setTabRef(el, i)"
          type="button"
          class="relative z-10 flex h-6 min-w-0 items-center gap-1.5 rounded-full px-3 text-[13px] transition-colors duration-150"
          :class="
            activeTabId === tab.id
              ? 'font-medium text-[var(--ink)]'
              : 'text-[var(--muted)] hover:text-[var(--ink)]'
          "
          @click="emit('tabSelect', tab.id)"
        >
          <span class="min-w-0 truncate">{{ tab.name }}</span>
          <X
            class="size-3 shrink-0 rounded-full p-px opacity-50 transition-opacity hover:bg-black/10 hover:opacity-100 dark:hover:bg-white/15"
            @click.stop="emit('tabClose', tab.id)"
          />
        </button>
      </div>
    </div>

    <!-- RIGHT: display options + window controls -->
    <div
      class="flex shrink-0 items-center gap-0.5 border-l border-[var(--line)] pl-2 pr-0.5"
      data-tauri-no-drag
    >
      <!-- display: font + theme -->
      <div
        class="flex items-center gap-0.5 pr-1.5"
        :title="`字号：${currentFontLabel()} · 主题：${currentThemeLabel()}`"
      >
        <button
          type="button"
          class="flex h-7 w-7 items-center justify-center rounded-md text-[var(--muted)] transition-colors hover:bg-black/5 hover:text-[var(--ink)] dark:hover:bg-white/10 disabled:opacity-30"
          :disabled="fontSize <= 0"
          :title="`字号：${currentFontLabel()}（点击减小）`"
          @click="bumpFont(-1)"
        >
          <span class="text-[11px] font-medium leading-none">A−</span>
        </button>
        <button
          type="button"
          class="flex h-7 w-7 items-center justify-center rounded-md text-[var(--muted)] transition-colors hover:bg-black/5 hover:text-[var(--ink)] dark:hover:bg-white/10 disabled:opacity-30"
          :disabled="fontSize >= FONT_STEPS.length - 1"
          :title="`字号：${currentFontLabel()}（点击增大）`"
          @click="bumpFont(1)"
        >
          <span class="text-[14px] font-semibold leading-none">A+</span>
        </button>
        <button
          type="button"
          class="flex h-7 w-7 items-center justify-center rounded-md text-[var(--muted)] transition-colors hover:bg-black/5 hover:text-[var(--ink)] dark:hover:bg-white/10"
          :title="`主题：${currentThemeLabel()}（点击切换）`"
          @click="cycleTheme"
        >
          <Sun v-if="theme === 'light'" class="size-[15px]" />
          <Moon v-else-if="theme === 'dark'" class="size-[15px]" />
          <Monitor v-else class="size-[15px]" />
        </button>
        <button
          type="button"
          class="flex h-7 w-7 items-center justify-center rounded-md text-[var(--muted)] transition-colors hover:bg-black/5 hover:text-[var(--ink)] dark:hover:bg-white/10"
          title="设置"
          @click="openSettings"
        >
          <Settings class="size-[15px]" />
        </button>
      </div>

      <!-- window controls -->
      <div class="flex items-center gap-0.5 pl-1">
        <button
          type="button"
          class="flex h-8 w-9 items-center justify-center rounded-md text-[var(--muted)] transition-colors hover:bg-black/5 hover:text-[var(--ink)] dark:hover:bg-white/10"
          aria-label="最小化"
          @click="minimize"
        >
          <Minus class="size-3.5" />
        </button>
        <button
          type="button"
          class="flex h-8 w-9 items-center justify-center rounded-md text-[var(--muted)] transition-colors hover:bg-black/5 hover:text-[var(--ink)] dark:hover:bg-white/10"
          :aria-label="isMaximized ? '还原' : '最大化'"
          @click="toggleMaximize"
        >
          <Square v-if="!isMaximized" class="size-3" />
          <Copy v-else class="size-3" />
        </button>
        <button
          type="button"
          class="flex h-8 w-9 items-center justify-center rounded-md text-[var(--muted)] transition-colors hover:bg-red-500/10 hover:text-red-500"
          aria-label="关闭"
          @click="closeWindow"
        >
          <X class="size-4" />
        </button>
      </div>
    </div>
  </div>

  <!-- Settings / app-info modal. Teleported to body so it isn't clipped
       by the titlebar's flex layout or data-tauri-drag-region. -->
  <Teleport to="body">
    <Transition name="modal-fade">
      <div
        v-if="showSettings"
        class="fixed inset-0 z-[100] flex items-center justify-center bg-black/45 p-4 backdrop-blur-sm dark:bg-black/65"
        role="dialog"
        aria-modal="true"
        aria-label="Interlocutor 设置"
        @click.self="closeSettings"
      >
        <div
          class="bg-[var(--surface)] text-[var(--ink)] flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-2xl border border-[var(--line)] shadow-2xl"
        >
          <!-- header -->
          <div class="flex shrink-0 items-center justify-between border-b border-[var(--line)] px-6 py-4">
            <div class="flex items-center gap-2.5">
              <div class="flex size-7 items-center justify-center rounded-lg bg-[var(--accent-soft)] text-[var(--accent)]">
                <Settings class="size-4" />
              </div>
              <h2 class="text-base font-semibold">Interlocutor · 设置</h2>
            </div>
            <button
              type="button"
              class="text-[var(--muted)] flex h-7 w-7 items-center justify-center rounded-md transition-colors hover:bg-black/5 hover:text-[var(--ink)] dark:hover:bg-white/10"
              title="关闭 (Esc)"
              @click="closeSettings"
            >
              <X class="size-4" />
            </button>
          </div>

          <!-- body -->
          <div class="min-h-0 flex-1 overflow-y-auto px-6 py-5">
            <!-- display -->
            <section class="mb-5">
              <h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wider text-[var(--muted)]">显示</h3>
              <div class="rounded-xl border border-[var(--line)] p-4">
                <div class="mb-4">
                  <div class="mb-2 text-sm font-medium">主题</div>
                  <div class="flex gap-1">
                    <button
                      v-for="opt in (['light', 'dark', 'auto'] as const)"
                      :key="opt"
                      type="button"
                      class="flex-1 rounded-md border px-3 py-1.5 text-[13px] transition-colors"
                      :class="
                        theme === opt
                          ? 'border-[var(--accent)] bg-[var(--accent-soft)] text-[var(--accent)]'
                          : 'border-[var(--line)] hover:bg-black/5 dark:hover:bg-white/10'
                      "
                      @click="theme = opt; applyDisplay()"
                    >
                      <span class="inline-flex items-center justify-center gap-1.5">
                        <Sun v-if="opt === 'light'" class="size-3.5" />
                        <Moon v-else-if="opt === 'dark'" class="size-3.5" />
                        <Monitor v-else class="size-3.5" />
                        {{ opt === 'light' ? '浅色' : opt === 'dark' ? '深色' : '自动' }}
                      </span>
                    </button>
                  </div>
                </div>
                <div>
                  <div class="mb-2 text-sm font-medium">字号</div>
                  <div class="flex gap-1">
                    <button
                      v-for="(step, i) in FONT_STEPS"
                      :key="step.label"
                      type="button"
                      class="flex-1 rounded-md border px-3 py-1.5 text-[13px] transition-colors"
                      :class="
                        fontSize === i
                          ? 'border-[var(--accent)] bg-[var(--accent-soft)] text-[var(--accent)]'
                          : 'border-[var(--line)] hover:bg-black/5 dark:hover:bg-white/10'
                      "
                      @click="fontSize = i; applyDisplay()"
                    >
                      {{ step.label }}
                      <span class="ml-1 text-[10px] opacity-60">{{ step.px }}</span>
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <!-- storage -->
            <section class="mb-5">
              <h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wider text-[var(--muted)]">数据</h3>
              <div class="rounded-xl border border-[var(--line)] p-4">
                <div class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-[13px]">
                  <div class="text-[var(--muted)]">默认保存位置</div>
                  <div class="font-mono text-[12px]">~/Documents/Interlocutor</div>
                  <div class="text-[var(--muted)]">支持格式</div>
                  <div class="flex flex-wrap items-center gap-1.5">
                    <code class="rounded bg-[var(--inline-code-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--inline-code-color)]">.md</code>
                    <code class="rounded bg-[var(--inline-code-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--inline-code-color)]">.markdown</code>
                    <code class="rounded bg-[var(--inline-code-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--inline-code-color)]">.mdown</code>
                    <code class="rounded bg-[var(--inline-code-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--inline-code-color)]">.txt</code>
                  </div>
                  <div class="text-[var(--muted)]">导出</div>
                  <div class="flex flex-wrap items-center gap-1.5">
                    <code class="rounded bg-[var(--inline-code-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--inline-code-color)]">HTML</code>
                    <code class="rounded bg-[var(--inline-code-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--inline-code-color)]">Word</code>
                    <code class="rounded bg-[var(--inline-code-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--inline-code-color)]">PDF</code>
                    <code class="rounded bg-[var(--inline-code-bg)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--inline-code-color)]">PNG</code>
                  </div>
                </div>
              </div>
            </section>

            <!-- credits / 快捷键 -->
            <section>
              <h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wider text-[var(--muted)]">快捷键</h3>
              <div class="rounded-xl border border-[var(--line)] p-3">
                <div class="grid grid-cols-[1fr_auto] gap-x-6 gap-y-1.5 text-[13px]">
                  <div class="text-[var(--muted)]">新建</div>
                  <kbd class="rounded border border-[var(--line)] bg-[var(--surface-2)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--muted)]">Ctrl N</kbd>
                  <div class="text-[var(--muted)]">打开</div>
                  <kbd class="rounded border border-[var(--line)] bg-[var(--surface-2)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--muted)]">Ctrl O</kbd>
                  <div class="text-[var(--muted)]">保存</div>
                  <kbd class="rounded border border-[var(--line)] bg-[var(--surface-2)] px-1.5 py-0.5 font-mono text-[11px] text-[var(--muted)]">Ctrl S</kbd>
                </div>
              </div>
            </section>
          </div>

          <!-- footer -->
          <div class="flex shrink-0 items-center justify-end gap-2 border-t border-[var(--line)] bg-[var(--surface-2)] px-6 py-3">
            <button
              type="button"
              class="rounded-md bg-[var(--accent)] px-4 py-1.5 text-[13px] font-medium text-white transition-colors hover:bg-[var(--accent-hover)]"
              @click="closeSettings"
            >
              完成
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.18s ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}
.modal-fade-enter-active > div,
.modal-fade-leave-active > div {
  transition: transform 0.18s ease;
}
.modal-fade-enter-from > div,
.modal-fade-leave-to > div {
  transform: scale(0.97);
}
</style>
