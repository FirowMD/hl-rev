<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import ViewFunctionConstructor from "./ViewFunctionConstructor.svelte";
  import ViewTypeConstructor from "./ViewTypeConstructor.svelte";
  import ViewGlobalConstructor from "./ViewGlobalConstructor.svelte";
  import ViewNativeConstructor from "./ViewNativeConstructor.svelte";
  import ViewConstantConstructor from "./ViewConstantConstructor.svelte";
  import ViewStringConstructor from "./ViewStringConstructor.svelte";
  import ViewIntConstructor from "./ViewIntConstructor.svelte";
  import ViewFloatConstructor from "./ViewFloatConstructor.svelte";

  const dispatch = createEventDispatcher();

  let activeTab = $state("functions");

  // Edit state for each constructor type
  let functionEditIndex = $state<number | null>(null);
  let functionModalMode = $state<'edit' | 'create'>('create');
  
  let typeEditIndex = $state<number | null>(null);
  let typeModalMode = $state<'edit' | 'create'>('create');
  
  let globalEditIndex = $state<number | null>(null);
  let globalModalMode = $state<'edit' | 'create'>('create');
  
  let nativeEditIndex = $state<number | null>(null);
  let nativeModalMode = $state<'edit' | 'create'>('create');
  
  let constantEditIndex = $state<number | null>(null);
  let constantModalMode = $state<'edit' | 'create'>('create');

  let stringEditIndex = $state<number | null>(null);
  let stringModalMode = $state<'edit' | 'create'>('create');
  
  let intEditIndex = $state<number | null>(null);
  let intModalMode = $state<'edit' | 'create'>('create');
  
  let floatEditIndex = $state<number | null>(null);
  let floatModalMode = $state<'edit' | 'create'>('create');

  const tabs = [
    { 
      id: 'functions', 
      label: 'Functions'
    },
    { 
      id: 'types', 
      label: 'Types'
    },
    { 
      id: 'globals', 
      label: 'Globals'
    },
    { 
      id: 'natives', 
      label: 'Natives'
    },
    { 
      id: 'constants', 
      label: 'Constants'
    },
    { 
      id: 'strings', 
      label: 'Strings'
    },
    { 
      id: 'ints', 
      label: 'Integers'
    },
    { 
      id: 'floats', 
      label: 'Floats'
    }
  ];

  // Handle edit requests from external components
  export function editFunction(index: number) {
    functionEditIndex = index;
    functionModalMode = 'edit';
    activeTab = 'functions';
  }

  export function editType(index: number) {
    typeEditIndex = index;
    typeModalMode = 'edit';
    activeTab = 'types';
  }

  export function editGlobal(index: number) {
    globalEditIndex = index;
    globalModalMode = 'edit';
    activeTab = 'globals';
  }

  export function editNative(index: number) {
    nativeEditIndex = index;
    nativeModalMode = 'edit';
    activeTab = 'natives';
  }

  export function editConstant(index: number) {
    constantEditIndex = index;
    constantModalMode = 'edit';
    activeTab = 'constants';
  }

  export function editString(index: number) {
    stringEditIndex = index;
    stringModalMode = 'edit';
    activeTab = 'strings';
  }

  export function editInt(index: number) {
    intEditIndex = index;
    intModalMode = 'edit';
    activeTab = 'ints';
  }

  export function editFloat(index: number) {
    floatEditIndex = index;
    floatModalMode = 'edit';
    activeTab = 'floats';
  }

  // Reset edit state when switching tabs
  function switchTab(tabId: string) {
    activeTab = tabId;
    // Reset all edit states when switching tabs
    functionEditIndex = null;
    functionModalMode = 'create';
    typeEditIndex = null;
    typeModalMode = 'create';
    globalEditIndex = null;
    globalModalMode = 'create';
    nativeEditIndex = null;
    nativeModalMode = 'create';
    constantEditIndex = null;
    constantModalMode = 'create';
    stringEditIndex = null;
    stringModalMode = 'create';
    intEditIndex = null;
    intModalMode = 'create';
    floatEditIndex = null;
    floatModalMode = 'create';
  }

  // Forward events from child constructors
  function forwardEvent(eventName: string, eventData: any) {
    dispatch(eventName, eventData);
  }
</script>

<div class="subnav-layout grid h-full min-h-0 grid-cols-[13rem_minmax(0,1fr)] overflow-hidden rounded-sm">
  <nav class="subnav-list flex flex-col gap-1 overflow-y-auto border-r border-surface-700/70 p-2" aria-label="Constructor entities">
    {#each tabs as tab}
      <button
        class="subnav-tab w-full rounded px-3 py-2 text-left text-xs font-medium {activeTab === tab.id ? 'subnav-tab-active' : ''}"
        onclick={() => switchTab(tab.id)}
        aria-current={activeTab === tab.id ? 'page' : undefined}
      >
        {tab.label}
      </button>
    {/each}
  </nav>

  <div class="min-h-0 overflow-y-auto p-3">
    <div class="constructor-editor mx-auto max-w-5xl">
      {#if activeTab === 'functions'}
        <ViewFunctionConstructor
          bind:modalMode={functionModalMode}
          bind:editFunctionIndex={functionEditIndex}
          on:save={(e) => forwardEvent('functionSave', e.detail)}
          on:edit={(e) => forwardEvent('functionEdit', e.detail)}
        />
      {:else if activeTab === 'types'}
        <ViewTypeConstructor
          bind:modalMode={typeModalMode}
          bind:editTypeIndex={typeEditIndex}
          on:save={(e) => forwardEvent('typeSave', e.detail)}
          on:edit={(e) => forwardEvent('typeEdit', e.detail)}
        />
      {:else if activeTab === 'globals'}
        <ViewGlobalConstructor
          bind:modalMode={globalModalMode}
          bind:editGlobalIndex={globalEditIndex}
          on:save={(e) => forwardEvent('globalSave', e.detail)}
          on:edit={(e) => forwardEvent('globalEdit', e.detail)}
        />
      {:else if activeTab === 'natives'}
        <ViewNativeConstructor
          bind:modalMode={nativeModalMode}
          bind:editNativeIndex={nativeEditIndex}
          on:save={(e) => forwardEvent('nativeSave', e.detail)}
          on:edit={(e) => forwardEvent('nativeEdit', e.detail)}
        />
      {:else if activeTab === 'constants'}
        <ViewConstantConstructor
          bind:modalMode={constantModalMode}
          bind:editConstantIndex={constantEditIndex}
          on:save={(e) => forwardEvent('constantSave', e.detail)}
          on:edit={(e) => forwardEvent('constantEdit', e.detail)}
        />
      {:else if activeTab === 'strings'}
        <ViewStringConstructor
          bind:modalMode={stringModalMode}
          bind:editStringIndex={stringEditIndex}
          on:save={(e) => forwardEvent('stringSave', e.detail)}
          on:edit={(e) => forwardEvent('stringEdit', e.detail)}
        />
      {:else if activeTab === 'ints'}
        <ViewIntConstructor
          bind:modalMode={intModalMode}
          bind:editIntIndex={intEditIndex}
          on:save={(e) => forwardEvent('intSave', e.detail)}
          on:edit={(e) => forwardEvent('intEdit', e.detail)}
        />
      {:else if activeTab === 'floats'}
        <ViewFloatConstructor
          bind:modalMode={floatModalMode}
          bind:editFloatIndex={floatEditIndex}
          on:save={(e) => forwardEvent('floatSave', e.detail)}
          on:edit={(e) => forwardEvent('floatEdit', e.detail)}
        />
      {/if}
    </div>
  </div>
</div>
