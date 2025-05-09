<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';
  import type { AIDecompilation, BytecodeItemSelectedEvent } from './types';

  let decompilerTitle = "Decompiler";
  let functionName = "";
  let decompiledCode = "";
  let decompiledCodeOptimized: string[] = [];
  let decompiledCodeLineNumber = 0;

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;
      const itemType = ev.detail.type;

      if (itemType === "function" || itemType === "class") {
        functionName = ev.detail.name;
        await updateDecompiler();
      }
    } catch (error) {
      console.log("Error fetching decompiled info:", error);
    }
  }

  async function updateDecompiler() {
    try {
      console.log("Updating decompiler for function:", functionName);
      
      const aiResult = await invoke("get_ai_decompilation", { 
        functionName 
      }) as AIDecompilation | null;

      console.log("AI decompilation result:", aiResult);

      if (aiResult) {
        console.log("Using AI decompilation for", functionName);
        decompiledCode = aiResult.result;
      } else {
        console.log("Using original decompilation for", functionName);
        const response = await invoke("get_decompiled_info") as string;
        if (response !== null) {
          decompiledCode = response;
        }
      }
      decompiledCodeOptimized = decompiledCode.split("\n");
      decompiledCodeLineNumber = decompiledCodeOptimized.length;
    } catch (error) {
      console.error("Error updating decompiler:", error);
    }
  }

  onMount(() => {
    updateDecompiler();
    window.addEventListener("ai-decompilation-replaced", async (e: Event) => {
      const ev = e as CustomEvent<{functionName: string, result: string}>;
      if (ev.detail.functionName === functionName) {
        console.log("Updating decompiler with AI result for", functionName);
        decompiledCode = ev.detail.result;
      }
    });

    window.addEventListener("ai-decompilation-removed", async (e: Event) => {
      const ev = e as CustomEvent<{functionName: string}>;
      if (ev.detail.functionName === functionName) {
        console.log("AI decompilation removed, reverting to original for", functionName);
        await updateDecompiler();
      }
    });

    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);

    return () => {
      window.removeEventListener("ai-decompilation-replaced", bytecodeItemSelectedHandler);
      window.removeEventListener("ai-decompilation-removed", bytecodeItemSelectedHandler);
      window.removeEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    };
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="p-2 space-y-2 preset-outlined-surface-500 bg-surface-900 h-full">
    <header class="h-12">
      <div class="flex flex-row items-start justify-between">
        <h5 class="h5">
          {decompilerTitle}
        </h5>
        {#if functionName !== ""}
          <h5 class="h5 truncate">
            {functionName}
          </h5>
        {/if}
      </div>
    </header>
    <section class="p-2 h-[calc(100%-3rem)]">
      {#if decompiledCode.length > 80000}
        <VirtualList width="100%" height="100%" itemCount={decompiledCodeLineNumber} itemSize={25} overscanCount={200}>
          <div slot="item" let:index let:style {style}>
            <div class="w-full text-left truncate">
              <span>{decompiledCodeOptimized[index]}</span>
            </div>
          </div>
        </VirtualList>
      {:else}
        <div class="h-full bg-surface-800 overflow-y-auto p-4">
          <pre>{decompiledCode}</pre>
        </div>
      {/if}
    </section>
  </div>
</div>