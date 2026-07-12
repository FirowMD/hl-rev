<script lang="ts">
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

<div class="side-panel w-full h-full overflow-hidden rounded-sm flex flex-col">
  <div class="side-panel-header flex items-center gap-2 border-b border-surface-700/70 px-2 py-1">
    <span class="shrink-0 text-[0.65rem] font-semibold uppercase text-surface-400">Code</span>
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
