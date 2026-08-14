<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { emit, listen } from "@tauri-apps/api/event"
import { open, save } from "@tauri-apps/plugin-dialog"
import { LoaderCircle } from "@lucide/vue"
import WysiwygEditor from "./WysiwygEditor.vue"
import { buildHtmlDocument, renderMarkdown } from "@/lib/markdown"
import {
  addTab,
  getActiveTab,
  markActiveTabSaved,
  updateActiveContent,
} from "@/lib/file-store"

const activeTab = computed(() => getActiveTab())
const content = computed({
  get: () => activeTab.value?.content ?? "",
  set: (value: string) => {
    updateActiveContent(value)
  },
})
const busy = ref(false)
const status = ref("就绪")
// 1-indexed line / column from the editor's current cursor position.
// Updated by the WysiwygEditor via the `cursor` event.
const cursorLine = ref(1)
const cursorCol = ref(1)
const previewRef = ref<HTMLElement | null>(null)
const renderedHtml = ref(renderMarkdown(content.value))
// Theme/font are owned by TitleBar; nothing to do here.
let statusTimer: number | undefined
let renderTimer: number | undefined
let unlistenFileOpen: (() => void) | undefined

watch(content, () => {
  window.clearTimeout(renderTimer)
  renderTimer = window.setTimeout(() => {
    renderedHtml.value = renderMarkdown(content.value)
  }, 180)
})

function setStatus(message: string) {
  status.value = message
  window.clearTimeout(statusTimer)
  statusTimer = window.setTimeout(() => {
    status.value = "就绪"
  }, 2600)
}

function defaultExportName(extension: string) {
  const base = (activeTab.value?.name ?? "untitled").replace(/\.[^.]+$/, "") || "untitled"
  return `${base}.${extension}`
}

async function openFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Markdown", extensions: ["md", "markdown", "txt"] }],
    })
    if (typeof selected !== "string")
      return
    const text = await invoke<string>("read_text_file", { path: selected })
    addTab(selected.split(/[\\/]/).pop() ?? "untitled.md", selected, text)
    setStatus("已打开")
  } catch (error) {
    setStatus(`打开失败：${String(error)}`)
  }
}

async function openPath(path: string) {
  try {
    const text = await invoke<string>("read_text_file", { path })
    addTab(path.split(/[\\/]/).pop() ?? "untitled.md", path, text)
    setStatus("已打开")
  } catch (error) {
    setStatus(`打开失败：${String(error)}`)
  }
}

async function setDefaultApp() {
  try {
    await invoke("open_default_apps_settings")
    setStatus("请在系统设置中把默认应用设为 Interlocutor")
  } catch (error) {
    setStatus(`打开系统设置失败：${String(error)}`)
  }
}

function newFile() {
  addTab("untitled.md")
  setStatus("已新建")
}

async function saveFile() {
  try {
    const path = activeTab.value?.path ?? await save({
      defaultPath: activeTab.value?.name || "untitled.md",
      filters: [{ name: "Markdown", extensions: ["md"] }],
    })
    if (!path)
      return
    await invoke("save_text_file", { path, contents: content.value })
    markActiveTabSaved(path, path.split(/[\\/]/).pop() ?? "untitled.md")
    setStatus("已保存")
  } catch (error) {
    setStatus(`保存失败：${String(error)}`)
  }
}

function dataUrlToBytes(dataUrl: string) {
  const base64 = dataUrl.split(",")[1]
  const binary = window.atob(base64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}

async function exportHtml() {
  try {
    const path = await save({
      defaultPath: defaultExportName("html"),
      filters: [{ name: "HTML", extensions: ["html"] }],
    })
    if (!path)
      return
    await invoke("save_text_file", {
      path,
      contents: buildHtmlDocument(content.value),
    })
    setStatus("HTML 已导出")
  } catch (error) {
    setStatus(`HTML 导出失败：${String(error)}`)
  }
}

async function exportWord() {
  try {
    const path = await save({
      defaultPath: defaultExportName("docx"),
      filters: [{ name: "Word", extensions: ["docx"] }],
    })
    if (!path)
      return
    const { markdownToDocx } = await import("@/lib/markdown-to-docx")
    const bytes = await markdownToDocx(content.value)
    await invoke("save_binary_file", {
      path,
      contents: Array.from(bytes),
    })
    setStatus("Word 已导出")
  } catch (error) {
    setStatus(`Word 导出失败：${String(error)}`)
  }
}

async function exportPng() {
  try {
    busy.value = true
    const target = previewRef.value
    if (!target)
      return
    const { toPng } = await import("html-to-image")
    const dataUrl = await toPng(target, {
      pixelRatio: 2,
      backgroundColor: "#ffffff",
    })
    const path = await save({
      defaultPath: defaultExportName("png"),
      filters: [{ name: "PNG 图片", extensions: ["png"] }],
    })
    if (!path)
      return
    await invoke("save_binary_file", {
      path,
      contents: Array.from(dataUrlToBytes(dataUrl)),
    })
    setStatus("图片已导出")
  } catch (error) {
    setStatus(`图片导出失败：${String(error)}`)
  } finally {
    busy.value = false
  }
}

async function exportPdf() {
  try {
    busy.value = true
    const target = previewRef.value
    if (!target)
      return
    const { toCanvas } = await import("html-to-image")
    const { jsPDF } = await import("jspdf")
    const canvas = await toCanvas(target, {
      pixelRatio: 2,
      backgroundColor: "#ffffff",
    })
    const pdf = new jsPDF({
      orientation: "portrait",
      unit: "pt",
      format: "a4",
    })
    const margin = 32
    const pageWidth = pdf.internal.pageSize.getWidth()
    const pageHeight = pdf.internal.pageSize.getHeight()
    const contentWidth = pageWidth - margin * 2
    const contentHeight = pageHeight - margin * 2
    const imageHeight = canvas.height * (contentWidth / canvas.width)
    const imageData = canvas.toDataURL("image/png")
    const pageCount = Math.max(1, Math.ceil(imageHeight / contentHeight))

    for (let page = 0; page < pageCount; page += 1) {
      const offset = page * contentHeight
      pdf.addImage(
        imageData,
        "PNG",
        margin,
        margin - offset,
        contentWidth,
        imageHeight,
        undefined,
        "FAST",
      )
      if (page < pageCount - 1)
        pdf.addPage()
    }

    const path = await save({
      defaultPath: defaultExportName("pdf"),
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    })
    if (!path)
      return
    const blob = pdf.output("blob")
    const bytes = new Uint8Array(await blob.arrayBuffer())
    await invoke("save_binary_file", {
      path,
      contents: Array.from(bytes),
    })
    setStatus("PDF 已导出")
  } catch (error) {
    setStatus(`PDF 导出失败：${String(error)}`)
  } finally {
    busy.value = false
  }
}

function runMenuAction(id: string) {
  switch (id) {
    case "new":
      newFile()
      break
    case "open":
      openFile()
      break
    case "set-default":
      setDefaultApp()
      break
    case "save":
      saveFile()
      break
    case "export-html":
      exportHtml()
      break
    case "export-word":
      exportWord()
      break
    case "export-pdf":
      exportPdf()
      break
    case "export-png":
      exportPng()
      break
    case "about":
      setStatus("Interlocutor Markdown 编辑器 · 基于 Rust Markdown 内核")
      break
  }
}

function onKeydown(event: KeyboardEvent) {
  if (!event.ctrlKey && !event.metaKey)
    return
  const key = event.key.toLowerCase()
  if (key === "n") {
    event.preventDefault()
    newFile()
  } else if (key === "o") {
    event.preventDefault()
    openFile()
  } else if (key === "s") {
    event.preventDefault()
    saveFile()
  }
}

onMounted(async () => {
  window.addEventListener("keydown", onKeydown)
  try {
    unlistenFileOpen = await listen<string>("file-open", (event) => {
      openPath(event.payload)
    })
    await emit("file-open-ready")
  } catch {
    // Not running inside Tauri.
  }
})

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown)
  unlistenFileOpen?.()
  window.clearTimeout(statusTimer)
  window.clearTimeout(renderTimer)
})

defineExpose({
  runMenuAction,
})
</script>

<template>
  <div class="bg-[var(--bg)] flex h-full min-h-0 flex-col">
    <main class="bg-[var(--surface)] relative min-h-0 flex-1">
      <div class="absolute inset-0 overflow-hidden">
        <WysiwygEditor v-model="content" @cursor="(l, c) => (cursorLine = l, cursorCol = c)" />
      </div>

      <div
        aria-hidden="true"
        class="bg-[var(--surface)] pointer-events-none fixed top-0 left-[-10000px] w-[820px] p-8"
      >
        <article
          ref="previewRef"
          class="md-body"
          v-html="renderedHtml"
        />
      </div>

    </main>

    <footer
      class="bg-[var(--surface)] text-[var(--muted)] flex h-7 shrink-0 items-center gap-4 border-t border-[var(--line)] px-3 text-xs select-none"
    >
      <LoaderCircle v-if="busy" class="size-3.5 animate-spin" />
      <span>{{ status }}</span>
      <span
        class="text-[var(--ink-soft)] font-mono tabular-nums"
        :title="`光标在第 ${cursorLine} 行第 ${cursorCol} 列`"
      >第 {{ cursorLine }} 行 · 第 {{ cursorCol }} 列</span>
      <span class="flex-1" />
      <span>{{ content.length }} 个字符</span>
      <span>UTF-8</span>
      <span>CRLF</span>
    </footer>
  </div>
</template>
