<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { writable } from "svelte/store";
  import ViewExport from './Tools/ViewExport.svelte';
  import ViewFunctionRecognizer from './Tools/ViewFunctionRecognizer.svelte';
  import ViewImHexPattern from './Tools/ViewImHexPattern.svelte';
  import ViewImportExportJson from './Tools/ViewImportExportJson.svelte';
  import ViewMergeBytecode from './Tools/ViewMergeBytecode.svelte';

  let references: string[] = $state([]);
  let elementIndex: number | null = $state(null);

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
    },
    {
      id: 'merge-bytecode',
      label: 'Merge Bytecode',
      component: ViewMergeBytecode,
      description: 'Merge external .hl/.dat into current bytecode'
    }
  ];

  onMount(() => {
    loadSavedReferences();
  });
</script>

<div class="h-full min-h-0 flex flex-col gap-2">
  <section class="subnav-layout grid min-h-0 flex-1 grid-cols-[13rem_minmax(0,1fr)] overflow-hidden rounded-sm">
    <nav class="subnav-list flex flex-col gap-1 overflow-y-auto border-r border-surface-700/70 p-2" aria-label="Tools">
      {#each toolsTabs as tab}
        <button
          class="subnav-tab w-full rounded px-3 py-2 text-left text-xs font-medium { $activeToolsTab === tab.id ? 'subnav-tab-active' : ''}"
          onclick={() => activeToolsTab.set(tab.id)}
          title={tab.description}
          aria-current={$activeToolsTab === tab.id ? 'page' : undefined}
        >
          {tab.label}
        </button>
      {/each}
    </nav>

    <div class="min-h-0 overflow-y-auto p-3">
      {#each toolsTabs as tab}
        {#if $activeToolsTab === tab.id}
          {@const Component = tab.component}
          <div class="mx-auto max-w-5xl">
            <Component />
          </div>
        {/if}
      {/each}
    </div>
  </section>

  {#if references.length > 0}
    <div class="rounded-sm border border-surface-700/70 bg-surface-900/80 px-3 py-2 text-xs text-surface-300">
      References: {references.length}
      {#if elementIndex !== null}
        <span class="text-surface-500"> for #{elementIndex}</span>
      {/if}
    </div>
  {/if}
</div>
