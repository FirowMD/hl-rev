<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // Use $props() instead of export let in Svelte 5
  let { modalMode = $bindable("create"), editGlobalIndex = $bindable(null) }: {
    modalMode: "edit" | "create";
    editGlobalIndex: number | null;
  } = $props();

  const dispatch = createEventDispatcher();

  // Form state
  let globalType = $state("");

  // Available reference data
  let availableTypes = $state<{ idx: number, name: string }[]>([]);

  // UI State
  let activePopover = $state<string | null>(null);
  let popoverSearchQueries = $state<Record<string, string>>({});
  let lastEditIndex = $state<number | null>(null);

  // Load global data when editing
  $effect(() => {
    if (editGlobalIndex !== lastEditIndex) {
      lastEditIndex = editGlobalIndex;
      
      if (editGlobalIndex !== null) {
        resetForm();
        fetchGlobalToEdit(editGlobalIndex);
      } else {
        resetForm();
      }
    }
  });

  async function fetchGlobalToEdit(idx: number) {
    try {
      const global = await invoke("get_global_full_info", { index: idx });
      console.log("Loaded global for editing:", global);
      
      // global is a RefType, so we just need the index
      globalType = String(global.index || global);
      
    } catch (e) {
      console.error("Failed to fetch global:", e);
      resetForm();
    }
  }

  // Fetch all required reference data
  async function fetchAllData() {
    await fetchTypes();
  }

  async function fetchTypes() {
    try {
      const rawTypes: string[] = await invoke("get_type_list");
      availableTypes = rawTypes.map(t => {
        const m = t.match(/^(.*)@(\d+)$/);
        if (m) return { name: m[1].trim(), idx: Number(m[2]) };
        return { name: t, idx: -1 };
      });
    } catch (e) {
      availableTypes = [];
    }
  }

  // Popover management
  function openPopover(type: string, query: string = "") {
    activePopover = type;
    popoverSearchQueries[type] = query;
  }

  function closePopover() {
    activePopover = null;
    popoverSearchQueries = {};
  }

  // Filter functions
  function getFilteredTypes(query: string) {
    return availableTypes.filter(t =>
      t.name.toLowerCase().includes(query.toLowerCase()) ||
      String(t.idx).includes(query)
    );
  }

  // Form submission
  function saveGlobal() {
    const globalData = {
      global_type: Number(globalType)
    };

    if (modalMode === "edit" && editGlobalIndex !== null) {
      dispatch("edit", {
        globalIndex: editGlobalIndex,
        ...globalData
      });
    } else {
      dispatch("save", globalData);
    }
    resetForm();
  }

  function resetForm() {
    globalType = "";
  }

  // Validation
  function canSaveGlobal() {
    return globalType && !isNaN(Number(globalType));
  }

  // Handle escape key
  function handleEscape(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closePopover();
    }
  }

  // Handle outside clicks
  function handleDocumentClick(e: MouseEvent) {
    const target = e.target as Element;
    if (!target.closest('.popover-container')) {
      closePopover();
    }
  }

  onMount(() => {
    fetchAllData();
    document.addEventListener("keydown", handleEscape);
    document.addEventListener("click", handleDocumentClick);
    
    return () => {
      document.removeEventListener("keydown", handleEscape);
      document.removeEventListener("click", handleDocumentClick);
    };
  });
</script>

<svelte:window on:keydown={handleEscape} />

<div class="h-full bg-surface-50 dark:bg-surface-900 overflow-hidden">
  <div class="h-full flex flex-col">
    
    <!-- Header -->
    <div class="bg-white dark:bg-surface-800 border-b border-surface-200 dark:border-surface-700 px-6 py-4">
      <div class="flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-bold text-surface-900 dark:text-surface-100">
            {modalMode === "edit" ? "Edit Global" : "Create Global"}
          </h1>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            {modalMode === "edit" ? "Modify the selected global variable" : "Define a new global variable for the bytecode"}
          </p>
        </div>
        
        <!-- Save Button -->
        <button
          class="btn bg-success-500 hover:bg-success-600 text-white rounded-lg px-6 py-2 text-sm font-medium
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={saveGlobal}
          disabled={!canSaveGlobal()}
        >
          {modalMode === "edit" ? "Save Global" : "Create Global"}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-2xl mx-auto space-y-8">
        
        <!-- Global Configuration -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="mb-6">
            <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
              Global Variable Configuration
            </h2>
            <p class="text-sm text-surface-600 dark:text-surface-400">
              Global variables are static values accessible throughout the bytecode. Define the type for this global variable.
            </p>
          </div>
          
          <!-- Global Type Selection -->
          <div class="space-y-2">
            <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="global-type-select">
              Global Type *
            </label>
            <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
              Select the type that this global variable will hold
            </p>
            <div class="relative popover-container">
              <button
                type="button"
                id="global-type-select"
                class="input w-full text-left flex items-center justify-between
                  bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                  rounded-lg px-3 py-2 text-sm hover:border-primary-400 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                onclick={() => openPopover('globalType')}
              >
                <span class={globalType ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                  {globalType ? availableTypes.find(t => t.idx === +globalType)?.name ?? globalType : 'Select global type...'}
                </span>
                <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                </svg>
              </button>
              
              {#if activePopover === 'globalType'}
                <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                  <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                    <input
                      class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                        bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                        focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                      type="text"
                      placeholder="Search types..."
                      bind:value={popoverSearchQueries.globalType}
                    />
                  </div>
                  <div class="max-h-60 overflow-y-auto">
                    {#each getFilteredTypes(popoverSearchQueries.globalType || '') as type}
                      <button
                        class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                          text-surface-900 dark:text-surface-100 flex items-center justify-between"
                        onclick={() => { globalType = String(type.idx); closePopover(); }}
                      >
                        <span class="font-mono">{type.name}</span>
                        <span class="text-xs text-surface-500">@{type.idx}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </div>

        <!-- Additional Information -->
        <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-700 p-4">
          <div class="flex items-start gap-3">
            <div class="flex-shrink-0">
              <svg class="w-5 h-5 text-blue-600 dark:text-blue-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </div>
            <div>
              <h3 class="text-sm font-medium text-blue-800 dark:text-blue-200 mb-1">
                About Global Variables
              </h3>
              <div class="text-xs text-blue-700 dark:text-blue-300 space-y-1">
                <p>• Global variables are static storage accessible from any function</p>
                <p>• They maintain their values throughout program execution</p>
                <p>• Constants can be used to initialize global variables</p>
                <p>• The type determines what kind of values can be stored</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Validation Summary -->
        <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
          <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Global Status</h3>
          <div class="space-y-1 text-xs">
            <div class={`flex items-center gap-2 ${globalType ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {globalType ? '✓' : '✗'}
              </span>
              Global type: {globalType ? 'Selected' : 'Required'}
            </div>
            <div class="flex items-center gap-2 text-surface-600 dark:text-surface-400">
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                ℹ
              </span>
              Index: Will be auto-assigned when created
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>