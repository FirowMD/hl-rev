<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ViewFunctions from "../elems/ViewFunctions.svelte";
  import ViewTypes from "../elems/ViewTypes.svelte";
  import ViewFiles from "../elems/ViewFiles.svelte";
  import ViewNatives from "../elems/ViewNatives.svelte";

  let activeTab = 'functions';

  const tabs = [
    { id: 'functions', label: 'Functions', component: ViewFunctions },
    { id: 'types', label: 'Classes', component: ViewTypes },
    { id: 'files', label: 'Files', component: ViewFiles },
    { id: 'natives', label: 'Natives', component: ViewNatives }
  ];
</script>

<div class="w-full h-full bg-surface-950">
  <div class="flex border-b border-surface-700 overflow-x-auto">
    {#each tabs as tab}
      <button
        class="px-4 py-1 {activeTab === tab.id ? 'bg-surface-800 border-b-2 border-primary-500' : 'hover:bg-surface-800/50'}"
        onclick={() => activeTab = tab.id}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="px-2 py-1 h-[calc(100%-3rem)] overflow-hidden">
    {#each tabs as tab}
      {#if activeTab === tab.id}
        <svelte:component this={tab.component} />
      {/if}
    {/each}
  </div>
</div>