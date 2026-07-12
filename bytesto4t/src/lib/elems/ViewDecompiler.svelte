<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';

  interface BytecodeItemSelectedEvent {
    name: string;
    type: string;
  }

  let functionName = $state("");
  let decompiledCode = $state("");
  let decompiledCodeOptimized = $state<string[]>([]);
  let decompiledCodeLineNumber = $state(0);
  let isLargeCode = $derived(decompiledCode.length > 80000);

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;
      const itemType = ev.detail.type;

      if (itemType === "function" || itemType === "class") {
        functionName = ev.detail.name;
        await updateDecompiler();
      }
    } catch (error) {
    }
  }

  async function updateDecompiler() {
    try {
      const response = await invoke("get_decompiled_info") as string;
      decompiledCode = response;
      decompiledCodeOptimized = decompiledCode.split("\n");
      decompiledCodeLineNumber = decompiledCodeOptimized.length;
    } catch (error) {
      console.error("Error updating decompiler:", error);
    }
  }

  onMount(() => {
    updateDecompiler();
    
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);

    return () => {
      window.removeEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    };
  });
</script>

<div class="h-full min-h-0 overflow-hidden">
  <section class="code-view flex h-full min-h-0 flex-col overflow-hidden rounded-sm">
    <header class="code-view-toolbar flex min-h-10 shrink-0 items-center justify-between gap-3 px-3 py-2">
      <div class="flex min-w-0 items-center gap-2 text-xs">
        {#if functionName !== ""}
          <span class="truncate font-semibold text-surface-100">{functionName}</span>
        {:else}
          <span class="text-surface-500">No function selected</span>
        {/if}
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-hidden p-2">
      {#if isLargeCode}
        <VirtualList width="100%" height="100%" itemCount={decompiledCodeLineNumber} itemSize={25} overscanCount={200}>
          <div slot="item" let:index let:style {style}>
            <div class="w-full text-left truncate">
              <span>{decompiledCodeOptimized[index]}</span>
            </div>
          </div>
        </VirtualList>
      {:else}
        <div class="h-full bg-surface-800 overflow-y-auto p-4 rounded-container-token">
          <pre>{decompiledCode}</pre>
        </div>
      {/if}
    </div>
  </section>
</div>
