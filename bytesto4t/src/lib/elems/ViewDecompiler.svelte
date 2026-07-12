<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';
  
  interface AIDecompilation {
    function_name: string;
    result: string;
    timestamp: string;
    model: string;
  }

  interface BytecodeItemSelectedEvent {
    name: string;
    type: string;
  }

  let functionName = $state("");
  let decompiledCode = $state("");
  let originalCode = $state("");
  let decompiledCodeOptimized = $state<string[]>([]);
  let decompiledCodeLineNumber = $state(0);
  let isEditing = $state(false);
  let hasManualEdit = $state(false);
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
      console.log("Error fetching decompiled info:", error);
    }
  }

  async function updateDecompiler() {
    try {
      console.log("Updating decompiler for function:", functionName);
      
      const manualEdit = await invoke("get_ai_decompilation", { 
        functionName 
      }) as AIDecompilation | null;

      if (manualEdit && manualEdit.model === "manual") {
        console.log("Using manual edit for", functionName);
        decompiledCode = manualEdit.result;
        hasManualEdit = true;
        
        const originalResponse = await invoke("get_decompiled_info") as string;
        originalCode = originalResponse;
      } else if (manualEdit && manualEdit.model !== "manual") {
        console.log("Using AI decompilation for", functionName);
        decompiledCode = manualEdit.result;
        hasManualEdit = false;
        
        const originalResponse = await invoke("get_decompiled_info") as string;
        originalCode = originalResponse;
      } else {
        console.log("Using original decompilation for", functionName);
        const response = await invoke("get_decompiled_info") as string;
        decompiledCode = response;
        originalCode = response;
        hasManualEdit = false;
      }
      
      decompiledCodeOptimized = decompiledCode.split("\n");
      decompiledCodeLineNumber = decompiledCodeOptimized.length;
    } catch (error) {
      console.error("Error updating decompiler:", error);
    }
  }

  async function saveManualEdit() {
    if (!functionName || !decompiledCode) return;
    
    try {
      await invoke("save_ai_decompilation", {
        functionName: functionName,
        result: decompiledCode,
        model: "manual"
      });
      
      const currentEdit = isEditing;
      
      hasManualEdit = true;
      isEditing = false;
      
      if (currentEdit === isEditing) {
        isEditing = !isEditing;
        isEditing = false;
      }

      window.dispatchEvent(new CustomEvent('manual-decompilation-saved', {
        detail: {
          functionName: functionName,
          result: decompiledCode
        }
      }));

      console.log("Manual edit saved for", functionName);
    } catch (error) {
      console.error("Failed to save manual edit:", error);
      alert("Failed to save manual edit");
    }
  }

  async function reDecompile() {
    if (!functionName) return;
    
    try {
      if (hasManualEdit) {
        await invoke("remove_ai_decompilation", { functionName });
      }
      
      const originalResponse = await invoke("get_decompiled_info") as string;
      decompiledCode = originalResponse;
      originalCode = originalResponse;
      hasManualEdit = false;
      isEditing = false;
      
      decompiledCodeOptimized = decompiledCode.split("\n");
      decompiledCodeLineNumber = decompiledCodeOptimized.length;

      window.dispatchEvent(new CustomEvent('manual-decompilation-removed', {
        detail: { functionName }
      }));

      console.log("Reverted to original decompilation for", functionName);
    } catch (error) {
      console.error("Failed to revert to original:", error);
      alert("Failed to revert to original decompilation");
    }
  }

  function toggleEdit() {
    if (isEditing) {
      if (hasManualEdit) {
        invoke("get_ai_decompilation", { functionName })
          .then((result: unknown) => {
            const aiDecomp = result as AIDecompilation | null;
            if (aiDecomp && aiDecomp.model === "manual") {
              decompiledCode = aiDecomp.result;
            }
          });
      } else {
        decompiledCode = originalCode;
      }
      isEditing = false;
    } else {
      isEditing = true;
    }
    
    decompiledCodeOptimized = decompiledCode.split("\n");
    decompiledCodeLineNumber = decompiledCodeOptimized.length;
  }

  function handleCodeChange() {
    decompiledCodeOptimized = decompiledCode.split("\n");
    decompiledCodeLineNumber = decompiledCodeOptimized.length;
  }

  async function handleAIDecompilationReplaced(e: Event) {
    const ev = e as CustomEvent<{functionName: string, result: string}>;
    if (ev.detail.functionName === functionName) {
      console.log("AI decompilation replaced, updating decompiler for", functionName);
      await updateDecompiler();
    }
  }

  async function handleAIDecompilationRemoved(e: Event) {
    const ev = e as CustomEvent<{functionName: string}>;
    if (ev.detail.functionName === functionName) {
      console.log("AI decompilation removed, updating decompiler for", functionName);
      await updateDecompiler();
    }
  }

  onMount(() => {
    updateDecompiler();
    
    window.addEventListener("ai-decompilation-replaced", handleAIDecompilationReplaced);
    window.addEventListener("ai-decompilation-removed", handleAIDecompilationRemoved);
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);

    return () => {
      window.removeEventListener("ai-decompilation-replaced", handleAIDecompilationReplaced);
      window.removeEventListener("ai-decompilation-removed", handleAIDecompilationRemoved);
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
          {#if hasManualEdit}
            <span class="badge preset-filled-warning-500 text-xs">Edited</span>
          {/if}
        {:else}
          <span class="text-surface-500">No function selected</span>
        {/if}
      </div>
      <div class="flex gap-2">
        <button
          type="button"
          class="btn preset-tonal-grain-raised-surface"
          onclick={toggleEdit}
          disabled={functionName === "" || decompiledCode === ""}
        >
          {isEditing ? 'Cancel' : 'Edit'}
        </button>
        {#if isEditing}
          <button type="button" class="btn preset-gradient-primary-grain" onclick={saveManualEdit}>
            Save
          </button>
        {/if}
        {#if hasManualEdit || isEditing}
          <button type="button" class="btn preset-tonal-grain-raised-surface" onclick={reDecompile}>
            Re-decompile
          </button>
        {/if}
      </div>
    </header>

    <div class="min-h-0 flex-1 overflow-hidden p-2">
      {#if isEditing}
        <textarea
          class="textarea bg-surface-800 w-full h-full font-mono text-sm resize-none focus:outline-none"
          bind:value={decompiledCode}
          oninput={handleCodeChange}
          placeholder="No decompiled code available"
        ></textarea>
      {:else if isLargeCode}
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
