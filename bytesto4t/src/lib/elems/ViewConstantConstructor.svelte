<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // Use $props() instead of export let in Svelte 5
  let { modalMode = $bindable("create"), editConstantIndex = $bindable(null) }: {
    modalMode: "edit" | "create";
    editConstantIndex: number | null;
  } = $props();

  const dispatch = createEventDispatcher();

  // Form state
  let globalReference = $state("");
  let fieldInitializers = $state<string[]>([]);

  // Available reference data
  let availableGlobals = $state<{ idx: number, value: string }[]>([]);

  // UI State
  let activePopover = $state<string | null>(null);
  let popoverSearchQueries = $state<Record<string, string>>({});
  let lastEditIndex = $state<number | null>(null);

  // Load constant data when editing
  $effect(() => {
    if (editConstantIndex !== lastEditIndex) {
      lastEditIndex = editConstantIndex;
      
      if (editConstantIndex !== null) {
        resetForm();
        fetchConstantToEdit(editConstantIndex);
      } else {
        resetForm();
      }
    }
  });

  async function fetchConstantToEdit(idx: number) {
    try {
      const constant = await invoke("get_constant_full_info", { index: idx }) as {
        global: number;
        fields: number[];
      };
      console.log("Loaded constant for editing:", constant);
      
      // Parse the serializable constant data and populate form fields
      globalReference = String(constant.global);
      fieldInitializers = constant.fields.map(String);
      
    } catch (e) {
      console.error("Failed to fetch constant:", e);
      resetForm();
    }
  }

  // Fetch all required reference data
  async function fetchAllData() {
    await fetchGlobals();
  }

  async function fetchGlobals() {
    try {
      const rawGlobals: string[] = await invoke("get_global_list");
      availableGlobals = rawGlobals.map(g => {
        const m = g.match(/^(.*)@(\d+)$/);
        if (m) return { value: m[1].trim(), idx: Number(m[2]) };
        return null;
      }).filter(Boolean) as { value: string; idx: number }[];
    } catch (e) {
      availableGlobals = [];
    }
  }

  // Field initializer management
  function addFieldInitializer() {
    fieldInitializers = [...fieldInitializers, ""];
  }

  function removeFieldInitializer(idx: number) {
    fieldInitializers = fieldInitializers.filter((_, i) => i !== idx);
  }

  function updateFieldInitializer(idx: number, value: string) {
    fieldInitializers = fieldInitializers.map((field, i) => i === idx ? value : field);
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
  function getFilteredGlobals(query: string) {
    return availableGlobals.filter(g =>
      g.value.toLowerCase().includes(query.toLowerCase()) ||
      String(g.idx).includes(query)
    );
  }

  // Form submission
  function saveConstant() {
    const constantData = {
      global: Number(globalReference),
      fields: fieldInitializers.map(f => Number(f)).filter(n => !isNaN(n))
    };

    if (modalMode === "edit" && editConstantIndex !== null) {
      dispatch("edit", {
        constantIndex: editConstantIndex,
        ...constantData
      });
    } else {
      dispatch("save", constantData);
    }
    resetForm();
  }

  function resetForm() {
    globalReference = "";
    fieldInitializers = [];
  }

  // Validation
  function canSaveConstant() {
    return globalReference && 
           !isNaN(Number(globalReference)) && 
           fieldInitializers.length > 0 && 
           fieldInitializers.every(f => f && !isNaN(Number(f)));
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
            {modalMode === "edit" ? "Edit Constant" : "Create Constant"}
          </h1>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            {modalMode === "edit" ? "Modify the selected constant initializer" : "Define a new constant initializer for global variables"}
          </p>
        </div>
        
        <!-- Save Button -->
        <button
          class="btn bg-success-500 hover:bg-success-600 text-white rounded-lg px-6 py-2 text-sm font-medium
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={saveConstant}
          disabled={!canSaveConstant()}
        >
          {modalMode === "edit" ? "Save Constant" : "Create Constant"}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-3xl mx-auto space-y-8">
        
        <!-- Constant Configuration -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="mb-6">
            <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
              Constant Initializer Configuration
            </h2>
            <p class="text-sm text-surface-600 dark:text-surface-400">
              Constants provide initial values for global variables. Select the global to initialize and define the field values.
            </p>
          </div>
          
          <!-- Global Reference Selection -->
          <div class="space-y-2 mb-6">
            <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="global-reference-select">
              Global Variable *
            </label>
            <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
              Select the global variable that this constant will initialize
            </p>
            <div class="relative popover-container">
              <button
                type="button"
                id="global-reference-select"
                class="input w-full text-left flex items-center justify-between
                  bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                  rounded-lg px-3 py-2 text-sm hover:border-primary-400 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                onclick={() => openPopover('globalReference')}
              >
                <span class={globalReference ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                  {globalReference ? availableGlobals.find(g => g.idx === +globalReference)?.value ?? globalReference : 'Select global variable...'}
                </span>
                <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                </svg>
              </button>
              
              {#if activePopover === 'globalReference'}
                <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                  <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                    <input
                      class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                        bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                        focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                      type="text"
                      placeholder="Search globals..."
                      bind:value={popoverSearchQueries.globalReference}
                    />
                  </div>
                  <div class="max-h-60 overflow-y-auto">
                    {#each getFilteredGlobals(popoverSearchQueries.globalReference || '') as global}
                      <button
                        class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                          text-surface-900 dark:text-surface-100 flex items-center justify-between"
                        onclick={() => { globalReference = String(global.idx); closePopover(); }}
                      >
                        <span class="font-mono">{global.value}</span>
                        <span class="text-xs text-surface-500">@{global.idx}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              {/if}
            </div>
          </div>
        </div>

        <!-- Field Initializers Section -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
                Field Initializers
              </h2>
              <p class="text-sm text-surface-600 dark:text-surface-400">
                Define the initial values for the fields of the global variable. Each field corresponds to a value index.
              </p>
            </div>
            <button
              class="btn btn-sm bg-primary-500 hover:bg-primary-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
              onclick={addFieldInitializer}
            >
              Add Field
            </button>
          </div>
          
          <div class="space-y-3">
            {#each fieldInitializers as field, idx}
              <div class="flex items-center gap-3 p-3 bg-surface-50 dark:bg-surface-700 rounded-lg border border-surface-200 dark:border-surface-600">
                <span class="text-sm font-medium text-surface-600 dark:text-surface-400 min-w-[60px]">
                  Field {idx}:
                </span>
                
                <div class="flex-1">
                  <input
                    class="input w-full bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                      rounded-lg px-3 py-2 text-sm text-surface-900 dark:text-surface-100
                      focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                    type="number"
                    min="0"
                    placeholder="Initializer value index"
                    value={field}
                    oninput={(e) => {
                      const target = e.target as HTMLInputElement;
                      if (target) updateFieldInitializer(idx, target.value);
                    }}
                  />
                </div>
                
                <button
                  class="btn btn-sm bg-error-500 hover:bg-error-600 text-white rounded-lg px-2 py-1.5"
                  onclick={() => removeFieldInitializer(idx)}
                >
                  Remove
                </button>
              </div>
            {/each}
            
            {#if fieldInitializers.length === 0}
              <div class="text-center py-8 text-surface-500 dark:text-surface-400">
                <p class="mb-2">No field initializers defined.</p>
                <p class="text-xs">Add field initializers to specify the initial values for the global variable.</p>
              </div>
            {/if}
          </div>
        </div>

        <!-- Information Panel -->
        <div class="bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-700 p-4">
          <div class="flex items-start gap-3">
            <div class="flex-shrink-0">
              <svg class="w-5 h-5 text-blue-600 dark:text-blue-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
              </svg>
            </div>
            <div>
              <h3 class="text-sm font-medium text-blue-800 dark:text-blue-200 mb-1">
                About Constant Initializers
              </h3>
              <div class="text-xs text-blue-700 dark:text-blue-300 space-y-1">
                <p>• Constants provide initial values for global variables during program startup</p>
                <p>• Each field index corresponds to a value in the constant pool (ints, floats, strings, etc.)</p>
                <p>• The number and type of fields must match the structure of the global variable type</p>
                <p>• Field order matters - they're applied in sequence to the global variable's structure</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Advanced Options -->
        <div class="bg-yellow-50 dark:bg-yellow-900/20 rounded-lg border border-yellow-200 dark:border-yellow-700 p-4">
          <div class="flex items-start gap-3">
            <div class="flex-shrink-0">
              <svg class="w-5 h-5 text-yellow-600 dark:text-yellow-400 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"></path>
              </svg>
            </div>
            <div>
              <h3 class="text-sm font-medium text-yellow-800 dark:text-yellow-200 mb-1">
                Field Value Indices
              </h3>
              <div class="text-xs text-yellow-700 dark:text-yellow-300 space-y-1">
                <p>Field values reference indices in the constant pools:</p>
                <p>• <strong>Integers:</strong> Index into the ints array</p>
                <p>• <strong>Floats:</strong> Index into the floats array</p>
                <p>• <strong>Strings:</strong> Index into the strings array</p>
                <p>• <strong>Objects:</strong> May reference other constants or null (0)</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Validation Summary -->
        <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
          <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Constant Status</h3>
          <div class="space-y-1 text-xs">
            <div class={`flex items-center gap-2 ${globalReference ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {globalReference ? '✓' : '✗'}
              </span>
              Global reference: {globalReference ? 'Selected' : 'Required'}
            </div>
            <div class={`flex items-center gap-2 ${fieldInitializers.length > 0 && fieldInitializers.every(f => f && !isNaN(Number(f))) ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {fieldInitializers.length > 0 && fieldInitializers.every(f => f && !isNaN(Number(f))) ? '✓' : '✗'}
              </span>
              Field initializers: {fieldInitializers.length > 0 ? 
                (fieldInitializers.every(f => f && !isNaN(Number(f))) ? `${fieldInitializers.length} valid` : 'Some invalid values') : 
                'At least one required'}
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