<script lang="ts">
  import ViewStrings from "../elems/ViewStrings.svelte";
  import ViewGlobals from "../elems/ViewGlobals.svelte";
  import ViewConstants from "../elems/ViewConstants.svelte";
  import ViewInts from "../elems/ViewInts.svelte";
  import ViewFloats from "../elems/ViewFloats.svelte";
  import ViewBytes from "../elems/ViewBytes.svelte";
  import ViewHistory from "../elems/ViewHistory.svelte";

  let activeTab = 'strings';

  const tabs = [
    { id: 'history', label: 'History', component: ViewHistory },
    { id: 'strings', label: 'Strings', component: ViewStrings },
    { id: 'globals', label: 'Globals', component: ViewGlobals },
    { id: 'constants', label: 'Constants', component: ViewConstants },
    { id: 'ints', label: 'Ints', component: ViewInts },
    { id: 'floats', label: 'Floats', component: ViewFloats },
    { id: 'bytes', label: 'Bytes', component: ViewBytes }
  ];
</script>

<div class="side-panel w-full h-full overflow-hidden rounded-sm flex flex-col">
  <div class="side-panel-header flex items-center gap-2 border-b border-surface-700/70 px-2 py-1">
    <span class="shrink-0 text-[0.65rem] font-semibold uppercase text-surface-400">Data</span>
    <div class="flex min-w-0 flex-1 gap-1 overflow-x-auto">
      {#each tabs as tab}
        <button
          class="side-panel-tab h-7 shrink-0 rounded px-2 text-xs {activeTab === tab.id ? 'side-panel-tab-active' : ''}"
          onclick={() => activeTab = tab.id}
        >
          {tab.label}
        </button>
      {/each}
    </div>
  </div>

  <div class="min-h-0 flex-1 px-2 py-1 overflow-hidden">
    {#each tabs as tab}
      {#if activeTab === tab.id}
        <svelte:component this={tab.component} />
      {/if}
    {/each}
  </div>
</div>
