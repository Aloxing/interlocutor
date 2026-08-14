import { ref } from "vue"

export interface FileTab {
  id: number
  name: string
  path: string | null
  content: string
}

// Two editor surfaces: rich-text (Tiptap) for the default preview+edit
// experience, and a raw markdown `<textarea>` for users who want to
// see / edit the source directly. Toggled by the eye button in the
// titlebar.
export type EditorMode = "wysiwyg" | "source"

// Test hook: ?mode=source forces the source view on first load (e.g. for
// headless screenshots). Anything other than "source" falls back to the
// default wysiwyg mode.
const initialMode: EditorMode =
  new URLSearchParams(location.search).get("mode") === "source" ? "source" : "wysiwyg"
export const editorMode = ref<EditorMode>(initialMode)

const initialContent = `# 欢迎使用 Interlocutor

这是一个超轻量 Markdown 编辑器和阅读器。查看 [Interlocutor on GitHub](https://github.com/Aloxing/interlocutor) 了解更多。

## 功能

- 所见即所得编辑
- 代码块高亮
- 导出 HTML / Word / PDF / 图片

\`\`\`ts
const hello = (name: string) => \`Hello, \${name}!\`
console.log(hello("Interlocutor"))
\`\`\`
`

let nextId = 1

export const tabs = ref<FileTab[]>([
  {
    id: 1,
    name: "welcome.md",
    path: null,
    content: initialContent,
  },
])

export const activeTabId = ref(1)

export function getActiveTab(): FileTab | undefined {
  return tabs.value.find(tab => tab.id === activeTabId.value)
    ?? tabs.value[0]
}

export function addTab(
  name = "untitled.md",
  path: string | null = null,
  content = "",
): FileTab {
  const tab: FileTab = {
    id: nextId += 1,
    name,
    path,
    content,
  }
  tabs.value.push(tab)
  activeTabId.value = tab.id
  return tab
}

export function closeTab(id: number): boolean {
  const index = tabs.value.findIndex(tab => tab.id === id)
  if (index === -1)
    return false

  tabs.value.splice(index, 1)
  if (activeTabId.value === id) {
    activeTabId.value = tabs.value[Math.min(index, tabs.value.length - 1)]?.id ?? 0
  }
  return tabs.value.length === 0
}

export function updateActiveContent(content: string) {
  const tab = getActiveTab()
  if (tab)
    tab.content = content
}

export function updateActiveName(name: string) {
  const tab = getActiveTab()
  if (tab)
    tab.name = name
}

export function markActiveTabSaved(path: string, name: string) {
  const tab = getActiveTab()
  if (tab) {
    tab.path = path
    tab.name = name
  }
}
