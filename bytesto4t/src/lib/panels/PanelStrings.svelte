<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import ViewStrings from "../elems/ViewStrings.svelte";
  import ViewGlobals from "../elems/ViewGlobals.svelte";
  import ViewConstants from "../elems/ViewConstants.svelte";
  import ViewInts from "../elems/ViewInts.svelte";
  import ViewFloats from "../elems/ViewFloats.svelte";
  import ViewHistory from "../elems/ViewHistory.svelte";

  let activeTab = 'strings';

  const tabs = [
    { id: 'history', label: 'History', component: ViewHistory },
    { id: 'strings', label: 'Strings', component: ViewStrings },
    { id: 'globals', label: 'Globals', component: ViewGlobals },
    { id: 'constants', label: 'Constants', component: ViewConstants },
    { id: 'ints', label: 'Ints', component: ViewInts },
    { id: 'floats', label: 'Floats', component: ViewFloats }
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
