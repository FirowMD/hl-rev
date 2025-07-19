<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';
  
  // Define the types locally since they might not be imported correctly
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

  let decompilerTitle = "Decompiler";
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
      
      // First check for manual edits (marked with "manual" model)
      const manualEdit = await invoke("get_ai_decompilation", { 
        functionName 
      }) as AIDecompilation | null;

      if (manualEdit && manualEdit.model === "manual") {
        console.log("Using manual edit for", functionName);
        decompiledCode = manualEdit.result;
        hasManualEdit = true;
        
        // Get original code for re-decompile functionality
        const originalResponse = await invoke("get_decompiled_info") as string;
        originalCode = originalResponse;
      } else if (manualEdit && manualEdit.model !== "manual") {
        // This is an AI decompilation
        console.log("Using AI decompilation for", functionName);
        decompiledCode = manualEdit.result;
        hasManualEdit = false;
        
        // Get original code for re-decompile functionality
        const originalResponse = await invoke("get_decompiled_info") as string;
        originalCode = originalResponse;
      } else {
        // Use original decompilation
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
      
      hasManualEdit = true;
      isEditing = false;

      // Dispatch event to notify other components
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
      // Remove manual edit if it exists
      if (hasManualEdit) {
        await invoke("remove_ai_decompilation", { functionName });
      }
      
      // Get fresh original decompilation
      const originalResponse = await invoke("get_decompiled_info") as string;
      decompiledCode = originalResponse;
      originalCode = originalResponse;
      hasManualEdit = false;
      isEditing = false;
      
      // Update optimized arrays
      decompiledCodeOptimized = decompiledCode.split("\n");
      decompiledCodeLineNumber = decompiledCodeOptimized.length;

      // Dispatch event to notify other components
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
      // Cancel editing - revert to last saved state
      if (hasManualEdit) {
        // If there was a manual edit, restore it
        invoke("get_ai_decompilation", { functionName })
          .then((result: unknown) => {
            const aiDecomp = result as AIDecompilation | null;
            if (aiDecomp && aiDecomp.model === "manual") {
              decompiledCode = aiDecomp.result;
            }
          });
      } else {
        // No manual edit, restore original
        decompiledCode = originalCode;
      }
      isEditing = false;
    } else {
      // Enter edit mode
      isEditing = true;
    }
    
    // Update optimized array
    decompiledCodeOptimized = decompiledCode.split("\n");
    decompiledCodeLineNumber = decompiledCodeOptimized.length;
  }

  function handleCodeChange() {
    // Update optimized array when code changes
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
    
    // Listen for AI decompilation events to update state accordingly
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

<div class="h-full overflow-y-auto">
  <div class="p-2 space-y-2 preset-outlined-surface-500 bg-surface-900 h-full">
    <header class="h-12">
      <div class="flex flex-row items-start justify-between">
        <div class="flex items-center gap-2">
          <h5 class="h5">
            {decompilerTitle}
          </h5>
          {#if functionName !== ""}
            <h5 class="h5 truncate">
              {functionName}
            </h5>
            {#if hasManualEdit}
              <span class="badge preset-filled-warning-500 text-xs">Edited</span>
            {/if}
          {/if}
        </div>
        
        <div class="flex gap-2">
          <button 
            type="button" 
            class="btn preset-outlined-surface-500 bg-surface-800"
            onclick={toggleEdit}
          >
            {isEditing ? 'Cancel' : 'Edit'}
          </button>
          
          {#if isEditing}
            <button 
              type="button" 
              class="btn preset-outlined-surface-500 bg-surface-800"
              onclick={saveManualEdit}
            >
              Save
            </button>
          {/if}
          
          {#if hasManualEdit || isEditing}
            <button 
              type="button" 
              class="btn preset-outlined-surface-500 bg-surface-800"
              onclick={reDecompile}
            >
              Re-decompile
            </button>
          {/if}
        </div>
      </div>
    </header>

    <section class="p-2 h-[calc(100%-3rem)]">
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
        <div class="h-full bg-surface-800 overflow-y-auto p-4">
          <pre>{decompiledCode}</pre>
        </div>
      {/if}
    </section>
  </div>
</div>