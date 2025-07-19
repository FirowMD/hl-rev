<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // Use $props() instead of export let in Svelte 5
  let { modalMode = $bindable("create"), editStringIndex = $bindable(null) }: {
    modalMode: "edit" | "create";
    editStringIndex: number | null;
  } = $props();

  const dispatch = createEventDispatcher();

  // Form state
  let stringValue = $state("");

  // UI State
  let lastEditIndex = $state<number | null>(null);

  // Load string data when editing
  $effect(() => {
    if (editStringIndex !== lastEditIndex) {
      lastEditIndex = editStringIndex;
      
      if (editStringIndex !== null) {
        resetForm();
        fetchStringToEdit(editStringIndex);
      } else {
        resetForm();
      }
    }
  });

  async function fetchStringToEdit(idx: number) {
    try {
      const value = await invoke("get_string_full_info", { index: idx }) as string;
      stringValue = value;
    } catch (e) {
      console.error("Failed to fetch string:", e);
      resetForm();
    }
  }

  // Form submission
  function saveString() {
    const stringData = {
      value: stringValue
    };

    if (modalMode === "edit" && editStringIndex !== null) {
      dispatch("edit", {
        stringIndex: editStringIndex,
        ...stringData
      });
    } else {
      dispatch("save", stringData);
    }
    resetForm();
  }

  function resetForm() {
    stringValue = "";
  }

  // Validation
  function canSaveString() {
    return stringValue.length > 0;
  }

  // Handle escape key
  function handleEscape(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      resetForm();
    }
  }

  onMount(() => {
    document.addEventListener("keydown", handleEscape);
    
    return () => {
      document.removeEventListener("keydown", handleEscape);
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
            {modalMode === "edit" ? "Edit String" : "Create String"}
          </h1>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            {modalMode === "edit" ? "Modify the selected string constant" : "Add a new string constant to the bytecode"}
          </p>
        </div>
        
        <!-- Save Button -->
        <button
          class="btn bg-success-500 hover:bg-success-600 text-white rounded-lg px-6 py-2 text-sm font-medium
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={saveString}
          disabled={!canSaveString()}
        >
          {modalMode === "edit" ? "Save String" : "Create String"}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-2xl mx-auto space-y-8">
        
        <!-- String Configuration -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="mb-6">
            <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
              String Value Configuration
            </h2>
            <p class="text-sm text-surface-600 dark:text-surface-400">
              String constants are used throughout the bytecode for names, literals, and identifiers.
            </p>
          </div>
          
          <!-- String Value Input -->
          <div class="space-y-2">
            <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="string-value-input">
              String Value *
            </label>
            <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
              Enter the string content. This will be available as a constant in the bytecode.
            </p>
            <textarea
              id="string-value-input"
              class="input w-full bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                rounded-lg px-3 py-2 text-sm text-surface-900 dark:text-surface-100
                focus:border-primary-500 focus:ring-1 focus:ring-primary-500 resize-y min-h-[80px]"
              bind:value={stringValue}
              placeholder="Enter string value..."
              rows="3"
            />
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
                About String Constants
              </h3>
              <div class="text-xs text-blue-700 dark:text-blue-300 space-y-1">
                <p>• Strings are stored in the constant pool and referenced by index</p>
                <p>• Used for function names, type names, field names, and string literals</p>
                <p>• Index 0 is reserved for "no name" and should not be modified</p>
                <p>• Strings support Unicode characters and escape sequences</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Character Count -->
        <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
          <div class="flex items-center justify-between">
            <span class="text-sm font-medium text-surface-700 dark:text-surface-300">Character Count</span>
            <span class="text-sm text-surface-600 dark:text-surface-400">{stringValue.length} characters</span>
          </div>
          {#if stringValue.length > 1000}
            <div class="mt-2 text-xs text-warning-600 dark:text-warning-400">
              ⚠️ Long strings may impact bytecode size and performance
            </div>
          {/if}
        </div>

        <!-- Validation Summary -->
        <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
          <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">String Status</h3>
          <div class="space-y-1 text-xs">
            <div class={`flex items-center gap-2 ${stringValue.length > 0 ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {stringValue.length > 0 ? '✓' : '✗'}
              </span>
              String value: {stringValue.length > 0 ? `${stringValue.length} characters` : 'Required - cannot be empty'}
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