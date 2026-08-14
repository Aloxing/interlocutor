<script setup lang="ts">
import { onBeforeUnmount, ref, watch, nextTick } from "vue"
import { EditorContent, useEditor } from "@tiptap/vue-3"
import StarterKit from "@tiptap/starter-kit"
import CodeBlockLowlight from "@tiptap/extension-code-block-lowlight"
import Link from "@tiptap/extension-link"
import Placeholder from "@tiptap/extension-placeholder"
import Typography from "@tiptap/extension-typography"
import { common, createLowlight } from "lowlight"
import { marked } from "marked"
import TurndownService from "turndown"
import { editorMode } from "@/lib/file-store"

// One lowlight instance with all common languages registered.
const lowlight = createLowlight(common)

// marked → HTML (only the bits Tiptap/ProseMirror understand)
marked.setOptions({
  gfm: true,
  breaks: false,
})

// HTML → markdown (Tiptap's editor.getHTML() → us → update modelValue)
const turndown = new TurndownService({
  headingStyle: "atx",
  codeBlockStyle: "fenced",
  bulletListMarker: "-",
  emDelimiter: "*",
  strongDelimiter: "**",
})
// Tiptap renders code blocks as <pre><code class="language-xxx">…</code></pre>
// Make sure turndown keeps the language hint in the fence info string.
turndown.addRule("fencedCodeBlockWithLanguage", {
  filter: (node: HTMLElement) =>
    node.nodeName === "PRE"
    && node.firstChild?.nodeName === "CODE",
  replacement: (_content: string, node: HTMLElement) => {
    const code = node.firstChild as HTMLElement | null
    const className = code?.getAttribute("class") ?? ""
    const langMatch = className.match(/language-([\w+-]+)/)
    const lang = langMatch ? langMatch[1] : ""
    const text = code?.textContent ?? ""
    return `\n\n\`\`\`${lang}\n${text.replace(/\n$/, "")}\n\`\`\`\n\n`
  },
})
// Keep <mark> / underline-ish marks as plain text
turndown.addRule("stripMarkTags", {
  filter: ["mark"],
  replacement: (content: string) => content,
})

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void
  (e: "cursor", line: number, column: number): void
}>()

const editor = useEditor({
  content: "",
  extensions: [
    // Disable the default codeBlock so CodeBlockLowlight takes over.
    StarterKit.configure({
      codeBlock: false,
    }),
    CodeBlockLowlight.configure({ lowlight }),
    Link.configure({
      openOnClick: false,
      autolink: true,
      HTMLAttributes: {
        rel: "noopener noreferrer",
        target: "_blank",
      },
    }),
    Placeholder.configure({
      placeholder: "开始书写…",
    }),
    Typography,
  ],
  editorProps: {
    attributes: {
      class: "tiptap md-body",
    },
  },
  onCreate({ editor }) {
    // Editor is now ready — seed it with the initial markdown.
    const initial = props.modelValue
    if (initial) {
      const html = marked.parse(initial, { async: false }) as string
      editor.commands.setContent(html, { emitUpdate: false })
    }
  },
  onUpdate({ editor }) {
    if (!editor) return
    const html = editor.getHTML()
    const md = turndown.turndown(html)
    emit("update:modelValue", md)
  },
  onSelectionUpdate({ editor }) {
    // Reports the cursor's 1-indexed line + column to the parent so the
    // status bar can render it. We count newlines in the text content
    // before the selection; this is the rendered view's "line", which
    // matches what the user sees on screen.
    const { state } = editor
    const { from } = state.selection
    const textBefore = state.doc.textBetween(0, from, "\n", "\n")
    const newlines = textBefore.length === 0 ? 0 : textBefore.split("\n").length - 1
    const line = newlines + 1
    const lastNl = textBefore.lastIndexOf("\n")
    const column = lastNl === -1 ? from + 1 : from - lastNl
    emit("cursor", line, column)
  },
})

const sourceTextareaRef = ref<HTMLTextAreaElement | null>(null)
function reportSourceCursor() {
  const ta = sourceTextareaRef.value
  if (!ta) return
  const pos = ta.selectionStart
  const before = ta.value.slice(0, pos)
  const newlines = before.length === 0 ? 0 : before.split("\n").length - 1
  const line = newlines + 1
  const lastNl = before.lastIndexOf("\n")
  const column = lastNl === -1 ? pos + 1 : pos - lastNl
  emit("cursor", line, column)
}

// When the source textarea is (re)created, sync the cursor position
// to the status bar so the line number is correct immediately after
// switching modes.
watch(editorMode, async (mode) => {
  if (mode === "source") {
    await nextTick()
    reportSourceCursor()
  }
})

const hostRef = ref<HTMLElement | null>(null)
const scrollbarRef = ref<HTMLElement | null>(null)
const thumbRef = ref<HTMLElement | null>(null)
let scrollElement: HTMLElement | null = null
let scrollbarHideTimer: number | undefined
let resizeObserver: ResizeObserver | undefined
let contentObserver: MutationObserver | undefined

function showScrollbar() {
  const bar = scrollbarRef.value
  const el = scrollElement
  if (!bar || !el || el.scrollHeight <= el.clientHeight) return
  bar.classList.add("visible")
  window.clearTimeout(scrollbarHideTimer)
  scrollbarHideTimer = window.setTimeout(() => {
    if (!bar.classList.contains("keep-visible")) {
      bar.classList.remove("visible")
    }
  }, 650)
}

function keepScrollbarVisible() {
  const bar = scrollbarRef.value
  if (!bar) return
  bar.classList.add("visible")
}

function releaseScrollbar() {
  const bar = scrollbarRef.value
  const el = scrollElement
  if (!bar || !el) return
  bar.classList.remove("keep-visible")
  window.clearTimeout(scrollbarHideTimer)
  scrollbarHideTimer = window.setTimeout(() => {
    bar.classList.remove("visible")
  }, 650)
}

function updateScrollbar() {
  const bar = scrollbarRef.value
  const thumb = thumbRef.value
  const el = scrollElement
  if (!bar || !thumb || !el) return
  const { scrollTop, scrollHeight, clientHeight } = el
  const trackHeight = bar.clientHeight
  const hasOverflow = scrollHeight > clientHeight
  const thumbHeight = hasOverflow
    ? Math.max(36, (clientHeight / scrollHeight) * trackHeight)
    : trackHeight
  const maxScroll = Math.max(1, scrollHeight - clientHeight)
  const maxTrack = Math.max(1, trackHeight - thumbHeight)
  thumb.style.height = `${thumbHeight}px`
  thumb.style.transform = `translateY(${(scrollTop / maxScroll) * maxTrack}px)`
  if (!hasOverflow) bar.classList.remove("visible")
}

function handleScroll() {
  updateScrollbar()
  showScrollbar()
}

function detachScrollbar() {
  if (scrollElement) scrollElement.removeEventListener("scroll", handleScroll)
  scrollElement = null
  resizeObserver?.disconnect()
  resizeObserver = undefined
  contentObserver?.disconnect()
  contentObserver = undefined
  window.clearTimeout(scrollbarHideTimer)
}

function attachScrollbar(element: HTMLElement) {
  detachScrollbar()
  scrollElement = element
  element.addEventListener("scroll", handleScroll, { passive: true })
  resizeObserver = new ResizeObserver(() => updateScrollbar())
  if (hostRef.value) resizeObserver.observe(hostRef.value)
  contentObserver = new MutationObserver(() => updateScrollbar())
  contentObserver.observe(element, {
    childList: true,
    subtree: true,
    characterData: true,
  })
  updateScrollbar()
}

function startThumbDrag(event: PointerEvent) {
  const el = scrollElement
  const bar = scrollbarRef.value
  const thumb = thumbRef.value
  if (!el || !bar || !thumb) return
  event.preventDefault()
  bar.classList.add("keep-visible")
  bar.classList.add("visible")
  const startY = event.clientY
  const startScroll = el.scrollTop
  const maxScroll = Math.max(1, el.scrollHeight - el.clientHeight)
  const maxTrack = Math.max(1, bar.clientHeight - thumb.clientHeight)

  const onPointerMove = (moveEvent: PointerEvent) => {
    const ratio = maxScroll / maxTrack
    el.scrollTop = startScroll + (moveEvent.clientY - startY) * ratio
  }
  const onPointerUp = () => {
    window.removeEventListener("pointermove", onPointerMove)
    window.removeEventListener("pointerup", onPointerUp)
    releaseScrollbar()
  }
  window.addEventListener("pointermove", onPointerMove)
  window.addEventListener("pointerup", onPointerUp, { once: true })
}

// Push external changes (e.g. open a new file) into the editor.
watch(
  () => props.modelValue,
  (value) => {
    if (!editor.value) return
    const current = turndown.turndown(editor.value.getHTML())
    if (current === value) return
    const html = marked.parse(value, { async: false }) as string
    editor.value.commands.setContent(html, { emitUpdate: false })
  },
)

/**
 * Click-based escape from a code block (Tiptap uses ProseMirror under the
 * hood, same caveat as before: clicking the <pre>/<code> wrapper outside
 * of text nodes drops the cursor in a place where arrow keys get stuck).
 * We snap the cursor to the last character of the code block.
 */
function attachCodeBlockClickEscape(view: any) {
  if (!view?.dom) return
  const handler = (event: MouseEvent) => {
    if (event.button !== 0) return
    const target = event.target as HTMLElement | null
    if (!target || target.nodeType === Node.TEXT_NODE) return

    const codeEl = target.closest("pre, code")
    if (!codeEl || !view.dom.contains(codeEl)) return

    queueMicrotask(() => {
      if (!view || view.isDestroyed) return
      const { state } = view
      const { selection } = state
      if (!selection.empty) return

      const schema = state.schema
      let codeBlockType: any = null
      schema.nodes.forEach((node: any) => {
        if (codeBlockType) return
        const name = node.name ?? node.spec?.name
        if (node.spec?.content === "text"
          && node.spec?.group?.includes("block")
          && /code/i.test(name)) {
          codeBlockType = node
        }
      })
      if (!codeBlockType) return

      const $from = selection.$from
      if ($from.parent.type !== codeBlockType) return

      const isWrapperClick
        = target === codeEl
          || target === codeEl.parentElement
          || target.tagName === "PRE"
          || target.tagName === "CODE"
      if (!isWrapperClick) return

      const codeBlockPos = $from.before()
      const nodeEnd = codeBlockPos + $from.parent.nodeSize
      const lastChar = nodeEnd - 1
      if ($from.pos === lastChar) return

      event.preventDefault()
      const TextSelection = state.selection.constructor
      const tr = state.tr.setSelection(
        TextSelection.near(state.doc.resolve(lastChar)),
      )
      view.dispatch(tr)
      view.focus()
    })
  }
  view.dom.addEventListener("mousedown", handler)
  ;(view as any).__cbClickHandler = handler
}

// Once the editor is ready, wire up wysiwyg-only behavior (the code-block
// click-escape). The overlay scrollbar is attached separately below so it
// can also follow the source textarea.
watch(
  editor,
  (e) => {
    if (!e) return
    const view = e.view as any
    if (view?.dom) {
      attachCodeBlockClickEscape(view)
    }
  },
  { immediate: true },
)

// Overlay scrollbar — attach to whichever surface is active (tiptap in
// wysiwyg mode, the textarea in source mode). Re-runs whenever either
// changes; attachScrollbar detaches the previous target first.
function attachScrollbarForMode() {
  if (editorMode.value === "source") {
    const ta = sourceTextareaRef.value
    if (ta) attachScrollbar(ta)
    return
  }
  const view = editor.value?.view as any
  if (view?.dom) attachScrollbar(view.dom as HTMLElement)
}

watch(
  [editor, editorMode],
  () => {
    nextTick(attachScrollbarForMode)
  },
  { immediate: true, flush: "post" },
)

onBeforeUnmount(() => {
  detachScrollbar()
  const view = editor.value?.view as any
  const handler = view && (view as any).__cbClickHandler
  if (view?.dom && handler) view.dom.removeEventListener("mousedown", handler)
  editor.value?.destroy()
})
</script>

<template>
  <div ref="hostRef" class="relative h-full w-full">
    <div
      class="h-full w-full px-[10%] py-[10px]"
    >
      <textarea
        v-if="editorMode === 'source'"
        ref="sourceTextareaRef"
        :value="modelValue"
        class="md-source block h-full w-full resize-none border-0 bg-transparent outline-none"
        spellcheck="false"
        :placeholder="`# 标题\n\n开始书写 Markdown…`"
        @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
        @click="reportSourceCursor"
        @keyup="reportSourceCursor"
        @select="reportSourceCursor"
      />
      <EditorContent v-else :editor="editor" class="h-full w-full" />
    </div>
    <div
      ref="scrollbarRef"
      class="editor-scrollbar"
      aria-hidden="true"
      @pointerenter="keepScrollbarVisible"
      @pointerleave="releaseScrollbar"
    >
      <div
        ref="thumbRef"
        class="editor-scrollbar-thumb"
        @pointerdown="startThumbDrag"
      />
    </div>
  </div>
</template>
