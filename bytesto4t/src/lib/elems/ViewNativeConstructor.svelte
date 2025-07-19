<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // Use $props() instead of export let in Svelte 5
  let { modalMode = $bindable("create"), editNativeIndex = $bindable(null) }: {
    modalMode: "edit" | "create";
    editNativeIndex: number | null;
  } = $props();

  const dispatch = createEventDispatcher();

  // Form state
  let libraryName = $state("");
  let functionName = $state("");
  let signatureType = $state("");
  let functionIndex = $state("");

  // Available reference data
  let availableStrings = $state<{ idx: number, value: string }[]>([]);
  let availableTypes = $state<{ idx: number, name: string }[]>([]);

  // UI State
  let activePopover = $state<string | null>(null);
  let popoverSearchQueries = $state<Record<string, string>>({});
  let lastEditIndex = $state<number | null>(null);

  // Load native data when editing
  $effect(() => {
    if (editNativeIndex !== lastEditIndex) {
      lastEditIndex = editNativeIndex;
      
      if (editNativeIndex !== null) {
        resetForm();
        fetchNativeToEdit(editNativeIndex);
      } else {
        resetForm();
      }
    }
  });

  async function fetchNativeToEdit(idx: number) {
    try {
      const native = await invoke("get_native_full_info", { index: idx });
      console.log("Loaded native for editing:", native);
      
      // Parse the native data and populate form fields
      libraryName = String(native.lib || "");
      functionName = String(native.name || "");
      signatureType = String(native.t || "");
      functionIndex = String(native.findex || "");
      
    } catch (e) {
      console.error("Failed to fetch native:", e);
      resetForm();
    }
  }

  // Fetch all required reference data
  async function fetchAllData() {
    await Promise.all([
      fetchStrings(),
      fetchTypes()
    ]);
  }

  async function fetchStrings() {
    try {
      const rawStrings: string[] = await invoke("get_string_list");
      availableStrings = rawStrings.map((s) => {
        const m = s.match(/^(.*)@(\d+)$/);
        if (m) return { value: m[1].trim(), idx: Number(m[2]) };
        return null;
      }).filter(Boolean) as { value: string; idx: number }[];
    } catch (e) {
      availableStrings = [];
    }
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
  function getFilteredStrings(query: string) {
    return availableStrings.filter(s =>
      s.value.toLowerCase().includes(query.toLowerCase()) ||
      String(s.idx).includes(query)
    );
  }

  function getFilteredTypes(query: string) {
    // Filter to only show function and method types
    return availableTypes.filter(t => {
      const matchesQuery = t.name.toLowerCase().includes(query.toLowerCase()) ||
        String(t.idx).includes(query);
      
      // Only show function signatures
      const isFunctionType = t.name.includes("->") || 
        t.name.includes("fun") || 
        t.name.includes("method") ||
        t.name.startsWith("(");
      
      return matchesQuery && isFunctionType;
    });
  }

  // Form submission
  function saveNative() {
    const nativeData = {
      lib: libraryName,
      name: functionName,
      signature_type: Number(signatureType),
      findex: functionIndex ? Number(functionIndex) : undefined
    };

    if (modalMode === "edit" && editNativeIndex !== null) {
      dispatch("edit", {
        nativeIndex: editNativeIndex,
        ...nativeData
      });
    } else {
      dispatch("save", nativeData);
    }
    resetForm();
  }

  function resetForm() {
    libraryName = "";
    functionName = "";
    signatureType = "";
    functionIndex = "";
  }

  // Validation
  function canSaveNative() {
    return libraryName && 
           functionName && 
           signatureType && 
           !isNaN(Number(signatureType));
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
            {modalMode === "edit" ? "Edit Native" : "Create Native"}
          </h1>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            {modalMode === "edit" ? "Modify the selected native function binding" : "Define a new native function binding to external libraries"}
          </p>
        </div>
        
        <!-- Save Button -->
        <button
          class="btn bg-success-500 hover:bg-success-600 text-white rounded-lg px-6 py-2 text-sm font-medium
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={saveNative}
          disabled={!canSaveNative()}
        >
          {modalMode === "edit" ? "Save Native" : "Create Native"}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-3xl mx-auto space-y-8">
        
        <!-- Native Function Configuration -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="mb-6">
            <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
              Native Function Configuration
            </h2>
            <p class="text-sm text-surface-600 dark:text-surface-400">
              Native functions are bindings to external library functions. Define the library, function name, and signature.
            </p>
          </div>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            
            <!-- Library Name -->
            <div class="space-y-2">
              <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="library-name-select">
                Library Name *
              </label>
              <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
                The name of the external library (e.g., "std", "opengl", "?sdl")
              </p>
              <div class="relative popover-container">
                <button
                  type="button"
                  id="library-name-select"
                  class="input w-full text-left flex items-center justify-between
                    bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                    rounded-lg px-3 py-2 text-sm hover:border-primary-400 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                  onclick={() => openPopover('libraryName')}
                >
                  <span class={libraryName ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                    {libraryName ? availableStrings.find(s => s.idx === +libraryName)?.value ?? libraryName : 'Select library name...'}
                  </span>
                  <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                  </svg>
                </button>
                
                {#if activePopover === 'libraryName'}
                  <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                    <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                      <input
                        class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                          bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                          focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                        type="text"
                        placeholder="Search library names..."
                        bind:value={popoverSearchQueries.libraryName}
                      />
                    </div>
                    <div class="max-h-60 overflow-y-auto">
                      {#each getFilteredStrings(popoverSearchQueries.libraryName || '') as str}
                        <button
                          class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                            text-surface-900 dark:text-surface-100 flex items-center justify-between"
                          onclick={() => { libraryName = String(str.idx); closePopover(); }}
                        >
                          <span class="font-mono">{str.value}</span>
                          <span class="text-xs text-surface-500">@{str.idx}</span>
                        </button>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            </div>

            <!-- Function Name -->
            <div class="space-y-2">
              <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="function-name-select">
                Function Name *
              </label>
              <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
                The name of the function in the external library
              </p>
              <div class="relative popover-container">
                <button
                  type="button"
                  id="function-name-select"
                  class="input w-full text-left flex items-center justify-between
                    bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                    rounded-lg px-3 py-2 text-sm hover:border-primary-400 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                  onclick={() => openPopover('functionName')}
                >
                  <span class={functionName ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                    {functionName ? availableStrings.find(s => s.idx === +functionName)?.value ?? functionName : 'Select function name...'}
                  </span>
                  <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                  </svg>
                </button>
                
                {#if activePopover === 'functionName'}
                  <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                    <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                      <input
                        class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                          bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                          focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                        type="text"
                        placeholder="Search function names..."
                        bind:value={popoverSearchQueries.functionName}
                      />
                    </div>
                    <div class="max-h-60 overflow-y-auto">
                      {#each getFilteredStrings(popoverSearchQueries.functionName || '') as str}
                        <button
                          class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                            text-surface-900 dark:text-surface-100 flex items-center justify-between"
                          onclick={() => { functionName = String(str.idx); closePopover(); }}
                        >
                          <span class="font-mono">{str.value}</span>
                          <span class="text-xs text-surface-500">@{str.idx}</span>
                        </button>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            </div>

            <!-- Function Signature -->
            <div class="space-y-2 md:col-span-2">
              <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="signature-type-select">
                Function Signature *
              </label>
              <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
                The function signature type (must be a function or method type)
              </p>
              <div class="relative popover-container">
                <button
                  type="button"
                  id="signature-type-select"
                  class="input w-full text-left flex items-center justify-between
                    bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                    rounded-lg px-3 py-2 text-sm hover:border-primary-400 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                  onclick={() => openPopover('signatureType')}
                >
                  <span class={signatureType ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                    {signatureType ? availableTypes.find(t => t.idx === +signatureType)?.name ?? signatureType : 'Select function signature...'}
                  </span>
                  <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                  </svg>
                </button>
                
                {#if activePopover === 'signatureType'}
                  <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                    <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                      <input
                        class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                          bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                          focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                        type="text"
                        placeholder="Search function types..."
                        bind:value={popoverSearchQueries.signatureType}
                      />
                    </div>
                    <div class="max-h-60 overflow-y-auto">
                      {#each getFilteredTypes(popoverSearchQueries.signatureType || '') as type}
                        <button
                          class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                            text-surface-900 dark:text-surface-100 flex items-center justify-between"
                          onclick={() => { signatureType = String(type.idx); closePopover(); }}
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

            <!-- Function Index -->
            <div class="space-y-2 md:col-span-2">
              <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="function-index-input">
                Function Index
              </label>
              <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
                Leave empty to auto-assign the next available function index
              </p>
              <input
                id="function-index-input"
                class="input w-full bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                  rounded-lg px-3 py-2 text-sm text-surface-900 dark:text-surface-100
                  focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                type="number"
                min="0"
                bind:value={functionIndex}
                placeholder="Auto-assigned if empty"
              />
            </div>
          </div>
        </div>

        <!-- Library Types Information -->
        <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-700 p-4">
          <div class="flex items-start gap-3">
            <div class="flex-shrink-0">
              <svg class="w-5 h-5 text-blue-600 dark:text-blue-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </div>
            <div>
              <h3 class="text-sm font-medium text-blue-800 dark:text-blue-200 mb-1">
                Library Loading Types
              </h3>
              <div class="text-xs text-blue-700 dark:text-blue-300 space-y-1">
                <p><strong>Standard libraries:</strong> "std" - Core hashlink functions</p>
                <p><strong>System libraries:</strong> "opengl", "sdl", "ssl" - System-level bindings</p>
                <p><strong>Lazy loading:</strong> "?libname" - Libraries loaded on first use (prefix with ?)</p>
                <p><strong>Signature:</strong> Must be a function type defining arguments and return value</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Validation Summary -->
        <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
          <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Native Status</h3>
          <div class="space-y-1 text-xs">
            <div class={`flex items-center gap-2 ${libraryName ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {libraryName ? '✓' : '✗'}
              </span>
              Library name: {libraryName ? 'Selected' : 'Required'}
            </div>
            <div class={`flex items-center gap-2 ${functionName ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {functionName ? '✓' : '✗'}
              </span>
              Function name: {functionName ? 'Selected' : 'Required'}
            </div>
            <div class={`flex items-center gap-2 ${signatureType ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {signatureType ? '✓' : '✗'}
              </span>
              Function signature: {signatureType ? 'Selected' : 'Required'}
            </div>
            <div class="flex items-center gap-2 text-surface-600 dark:text-surface-400">
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                ℹ
              </span>
              Function index: {functionIndex || 'Will be auto-assigned'}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>