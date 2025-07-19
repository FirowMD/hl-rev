<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { writable } from "svelte/store";
  import ViewExport from './Tools/ViewExport.svelte';
  import ViewFunctionRecognizer from './Tools/ViewFunctionRecognizer.svelte';
  import ViewImHexPattern from './Tools/ViewImHexPattern.svelte';
  import ViewImportExportJson from './Tools/ViewImportExportJson.svelte';

  let references: string[] = $state([]);
  let elementIndex: number | null = $state(null);

  // Create a store for the active tools tab
  const activeToolsTab = writable('export');

  function parseReference(ref: string) {
    const [funcPart, pos, op] = ref.split('###');
    return { funcPart, pos, op };
  }

  async function loadSavedReferences() {
    try {
      const saved = await invoke<[number, string[]] | null>("get_saved_references");
      if (saved) {
        const [idx, refs] = saved;
        elementIndex = idx;
        references = refs;
      }
    } catch (error) {
      console.error("Failed to load saved references:", error);
    }
  }

  // Define the tools tabs
  const toolsTabs = [
    { 
      id: 'export', 
      label: 'Export', 
      component: ViewExport,
      description: 'Export functions, classes, files and bytecode'
    },
    { 
      id: 'function-recognizer', 
      label: 'Function Recognizer', 
      component: ViewFunctionRecognizer,
      description: 'Match function addresses with function signatures'
    },
    { 
      id: 'imhex-pattern', 
      label: 'ImHex Pattern', 
      component: ViewImHexPattern,
      description: 'Generate ImHex pattern files from selected classes'
    },
    { 
      id: 'import-export-json', 
      label: 'Import/Export JSON', 
      component: ViewImportExportJson,
      description: 'Import and export individual functions and classes as JSON'
    }
  ];

  onMount(() => {
    loadSavedReferences();
  });
</script>

<div class="h-full preset-outlined-surface-500 bg-surface-900 flex flex-col">
  <!-- Tools Tab Navigation -->
  <div class="flex border-b border-surface-700 bg-surface-800 truncate overflow-x-auto">
    {#each toolsTabs as tab}
      <button
        class="px-3 py-2 text-sm font-medium transition-colors whitespace-nowrap {$activeToolsTab === tab.id ? 'bg-surface-700 border-b-2 border-primary-500 text-primary-300' : 'hover:bg-surface-700/50 text-surface-300'}"
        onclick={() => activeToolsTab.set(tab.id)}
        title={tab.description}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <!-- Tools Content Area -->
  <div class="flex-1 overflow-y-auto">
    <div class="p-3">
      {#each toolsTabs as tab}
        {#if $activeToolsTab === tab.id}
          <div class="space-y-2">
            <!-- Tab description -->
            <div class="text-sm text-surface-400 border-l-2 border-primary-500 pl-3 mb-4">
              {tab.description}
            </div>
            
            <!-- Tab component -->
            <svelte:component this={tab.component} />
          </div>
        {/if}
      {/each}
    </div>
  </div>

  <!-- References section (if needed) -->
  {#if references.length > 0}
    <div class="border-t border-surface-700 p-3 bg-surface-800">
      <h6 class="text-sm font-semibold mb-2 text-surface-200">References</h6>
      <div class="text-xs text-surface-400">
        Element Index: {elementIndex ?? 'None'} | References: {references.length}
      </div>
    </div>
  {/if}
</div>