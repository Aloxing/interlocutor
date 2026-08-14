<script setup lang="ts">
import { ref } from "vue"
import TitleBar from "./components/TitleBar.vue"
import MarkdownWorkspace from "./components/MarkdownWorkspace.vue"
import { activeTabId, closeTab } from "@/lib/file-store"

const workspaceRef = ref<InstanceType<typeof MarkdownWorkspace> | null>(null)

function handleMenuAction(id: string) {
  workspaceRef.value?.runMenuAction(id)
}

function handleTabSelect(id: number) {
  activeTabId.value = id
}

async function handleTabClose(id: number) {
  const empty = closeTab(id)
  if (!empty)
    return
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window")
    await getCurrentWindow().close()
  } catch {
    // Not running inside Tauri.
  }
}
</script>

<template>
  <div
    class="bg-[var(--surface)] text-[var(--ink)] flex h-screen w-screen flex-col overflow-hidden rounded-[7px] border border-[var(--line)] shadow-sm"
  >
    <TitleBar
      @menu-action="handleMenuAction"
      @tab-select="handleTabSelect"
      @tab-close="handleTabClose"
    />
    <div class="relative min-h-0 flex-1">
      <MarkdownWorkspace ref="workspaceRef" />
    </div>
  </div>
</template>
