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

  // Active tab state
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

  // NEW: Edit state for string, int, float constructors
  let stringEditIndex = $state<number | null>(null);
  let stringModalMode = $state<'edit' | 'create'>('create');
  
  let intEditIndex = $state<number | null>(null);
  let intModalMode = $state<'edit' | 'create'>('create');
  
  let floatEditIndex = $state<number | null>(null);
  let floatModalMode = $state<'edit' | 'create'>('create');

  // Updated tab definitions
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

  // NEW: Edit methods for string, int, float
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

<div class="h-full bg-surface-50 dark:bg-surface-900 overflow-hidden">
  <div class="h-full flex">
    
    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col">

      <!-- Constructor Content -->
      <div class="flex-1 overflow-hidden">
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
    
    <!-- Right Sidebar - Tab Navigation -->
    <div class="w-80 bg-white dark:bg-surface-800 border-l border-surface-200 dark:border-surface-700 flex flex-col">
      <!-- Sidebar Header -->
      <div class="px-6 py-4 border-b border-surface-200 dark:border-surface-700">
        <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100">
          Element Types
        </h2>
      </div>
      
      <!-- Tab Navigation -->
      <div class="flex-1 overflow-y-auto p-4 sidebar-scroll">
        <div class="space-y-2">
          {#each tabs as tab}
            <button
              class="w-full flex items-center justify-between px-4 py-3 text-left rounded-lg
                {activeTab === tab.id 
                  ? 'bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-300 border-l-4 border-primary-600 shadow-sm' 
                  : 'text-surface-700 dark:text-surface-300 hover:bg-surface-50 dark:hover:bg-surface-700/50 hover:text-surface-900 dark:hover:text-surface-100'}"
              onclick={() => switchTab(tab.id)}
            >
              <div class="flex-1 min-w-0">
                <h3 class="font-semibold text-sm truncate">{tab.label}</h3>
              </div>
              {#if activeTab === tab.id}
                <div class="flex-shrink-0 w-2 h-2 rounded-full bg-primary-600 dark:bg-primary-400 ml-3"></div>
              {/if}
            </button>
          {/each}
        </div>
      </div>
      
      <!-- Sidebar Footer -->
      <div class="px-6 py-4 border-t border-surface-200 dark:border-surface-700">
        <div class="text-xs text-surface-500 dark:text-surface-400">
          <p>Right-click items in lists to access edit options</p>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .line-clamp-2 {
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.4;
  }
  
  /* Enhance the sidebar scroll area */
  .sidebar-scroll {
    scrollbar-width: thin;
    scrollbar-color: rgba(156, 163, 175, 0.5) transparent;
  }
  
  .sidebar-scroll::-webkit-scrollbar {
    width: 6px;
  }
  
  .sidebar-scroll::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .sidebar-scroll::-webkit-scrollbar-thumb {
    background-color: rgba(156, 163, 175, 0.5);
    border-radius: 3px;
  }
  
  .sidebar-scroll::-webkit-scrollbar-thumb:hover {
    background-color: rgba(156, 163, 175, 0.7);
  }
</style>