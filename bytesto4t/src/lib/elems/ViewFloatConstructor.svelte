<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // Use $props() instead of export let in Svelte 5
  let { modalMode = $bindable("create"), editFloatIndex = $bindable(null) }: {
    modalMode: "edit" | "create";
    editFloatIndex: number | null;
  } = $props();

  const dispatch = createEventDispatcher();

  // Form state
  let floatValue = $state<number | string>("");

  // UI State
  let lastEditIndex = $state<number | null>(null);
  let inputMode = $state<"decimal" | "scientific">("decimal");

  // Load float data when editing
  $effect(() => {
    if (editFloatIndex !== lastEditIndex) {
      lastEditIndex = editFloatIndex;
      
      if (editFloatIndex !== null) {
        resetForm();
        fetchFloatToEdit(editFloatIndex);
      } else {
        resetForm();
      }
    }
  });

  async function fetchFloatToEdit(idx: number) {
    try {
      const value = await invoke("get_float_full_info", { index: idx }) as number;
      // Format the value according to the current input mode
      floatValue = formatNumber(value);
    } catch (e) {
      console.error("Failed to fetch float:", e);
      resetForm();
    }
  }

  // Helper functions for number parsing and validation
  function parseInput(input: string): number | null {
    if (typeof input === "number") return input;
    
    const trimmed = input.trim();
    if (trimmed === "") return null;
    
    // HashLink bytecode doesn't support special IEEE 754 values at runtime
    // Remove special value parsing
    
    try {
      // Use parseFloat which handles both decimal and scientific notation
      const parsed = parseFloat(trimmed);
      
      // Check if the input was actually a valid number format
      // parseFloat can parse partial numbers, so we need to validate
      const stringified = parsed.toString();
      const normalizedInput = trimmed.replace(/^\+/, ''); // Remove leading +
      
      // For scientific notation, check if the format is valid
      if (trimmed.includes('e') || trimmed.includes('E')) {
        // Basic scientific notation validation
        const scientificPattern = /^[+-]?\d*\.?\d+[eE][+-]?\d+$/;
        if (!scientificPattern.test(normalizedInput)) {
          return null;
        }
      }
      
      // Reject special IEEE 754 values that HashLink VM doesn't support at runtime
      if (isNaN(parsed) || !isFinite(parsed)) {
        return null;
      }
      
      return parsed;
    } catch {
      return null;
    }
  }

  function formatNumber(value: number): string {
    // Only handle finite numbers (no special values)
    if (!isFinite(value)) {
      return "0"; // Fallback for any edge cases
    }
    
    switch (inputMode) {
      case "scientific":
        // For scientific mode, always use exponential notation
        return value.toExponential();
      default:
        // Decimal mode shows normal decimal representation
        return value.toString();
    }
  }

  // Computed values
  let parsedValue = $derived(typeof floatValue === "number" ? floatValue : parseInput(floatValue as string));
  let isValidFloat = $derived(parsedValue !== null);

  // Get float info for display
  function getFloatInfo(value: number) {
    if (value === 0) return { type: "Zero", description: "Zero value" };
    if (Math.abs(value) < Number.MIN_VALUE) return { type: "Subnormal", description: "Very small number" };
    if (Number.isInteger(value)) return { type: "Integer", description: "Whole number as float" };
    if (Math.abs(value) > 1e100) return { type: "Very Large", description: "Large floating-point number" };
    if (Math.abs(value) < 1e-100) return { type: "Very Small", description: "Small floating-point number" };
    return { type: "Normal Float", description: "Standard floating-point number" };
  }

  // Form submission
  function saveFloat() {
    const finalValue = typeof floatValue === "number" ? floatValue : parseInput(floatValue as string);
    
    if (finalValue === null) return;

    const floatData = {
      value: finalValue
    };

    if (modalMode === "edit" && editFloatIndex !== null) {
      dispatch("edit", {
        floatIndex: editFloatIndex,
        ...floatData
      });
    } else {
      dispatch("save", floatData);
    }
    resetForm();
  }

  function resetForm() {
    floatValue = "";
    inputMode = "decimal";
  }

  // Validation
  function canSaveFloat() {
    return isValidFloat;
  }

  // Handle input mode change
  function changeInputMode(mode: "decimal" | "scientific") {
    const currentParsed = parsedValue;
    inputMode = mode;
    
    // Only reformat if we have a valid current value
    if (currentParsed !== null && isFinite(currentParsed)) {
      floatValue = formatNumber(currentParsed);
    }
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
            {modalMode === "edit" ? "Edit Float" : "Create Float"}
          </h1>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            {modalMode === "edit" ? "Modify the selected float constant" : "Add a new 64-bit float constant to the bytecode"}
          </p>
        </div>
        
        <!-- Save Button -->
        <button
          class="btn bg-success-500 hover:bg-success-600 text-white rounded-lg px-6 py-2 text-sm font-medium
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={saveFloat}
          disabled={!canSaveFloat()}
        >
          {modalMode === "edit" ? "Save Float" : "Create Float"}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-2xl mx-auto space-y-8">
        
        <!-- Float Configuration -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="mb-6">
            <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
              Float Value Configuration
            </h2>
            <p class="text-sm text-surface-600 dark:text-surface-400">
              64-bit double-precision floating-point constants for decimal calculations.
            </p>
          </div>
          
          <!-- Input Mode Selection -->
          <div class="mb-4">
            <label class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">
              Input Format
            </label>
            <div class="flex gap-2">
              <button
                class="btn btn-sm {inputMode === 'decimal' ? 'bg-primary-500 text-white' : 'bg-surface-200 dark:bg-surface-600'} 
                  hover:bg-primary-600 rounded-lg px-3 py-1.5 text-sm"
                onclick={() => changeInputMode('decimal')}
              >
                Decimal
              </button>
              <button
                class="btn btn-sm {inputMode === 'scientific' ? 'bg-primary-500 text-white' : 'bg-surface-200 dark:bg-surface-600'} 
                  hover:bg-primary-600 rounded-lg px-3 py-1.5 text-sm"
                onclick={() => changeInputMode('scientific')}
              >
                Scientific
              </button>
            </div>
          </div>

          <!-- Float Value Input -->
          <div class="space-y-2">
            <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="float-value-input">
              Float Value *
            </label>
            <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
              {#if inputMode === "decimal"}
                Enter a 64-bit floating-point number in decimal format
              {:else if inputMode === "scientific"}
                Enter in scientific notation (e.g., 1.23e-4, 6.02e23, -2.5E10)
              {/if}
            </p>
            <input
              id="float-value-input"
              class="input w-full bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                rounded-lg px-3 py-2 text-sm text-surface-900 dark:text-surface-100
                focus:border-primary-500 focus:ring-1 focus:ring-primary-500 font-mono
                {!isValidFloat && floatValue !== '' ? 'border-error-500' : ''}"
              type="text"
              bind:value={floatValue}
              placeholder={inputMode === "scientific" ? "1.23e-4" : "3.14159"}
            />
            
            {#if parsedValue !== null && isValidFloat}
              {@const info = getFloatInfo(parsedValue)}
              <div class="mt-2 p-2 bg-success-50 dark:bg-success-900/20 rounded border border-success-200 dark:border-success-700">
                <div class="text-xs text-success-700 dark:text-success-300 space-y-1">
                  <div><strong>Value:</strong> {parsedValue}</div>
                  <div><strong>Type:</strong> {info.type}</div>
                  <div><strong>Description:</strong> {info.description}</div>
                  <div><strong>Scientific:</strong> {parsedValue.toExponential()}</div>
                  <div><strong>Fixed:</strong> {parsedValue.toFixed(6)}</div>
                </div>
              </div>
            {:else if floatValue !== "" && !isValidFloat}
              <div class="mt-2 p-2 bg-error-50 dark:bg-error-900/20 rounded border border-error-200 dark:border-error-700">
                <div class="text-xs text-error-700 dark:text-error-300">
                  {#if inputMode === "decimal"}
                    Invalid decimal format. Examples: 3.14159, -42.5, 0.001
                  {:else if inputMode === "scientific"}
                    Invalid scientific notation. Examples: 1.23e-4, 6.02e23, -2.5E10
                  {:else}
                    Invalid number format. Please check your input.
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        </div>

        <!-- Quick Values -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-3">Quick Values</h3>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
            {#each [
              { label: "Zero", value: 0 },
              { label: "One", value: 1 },
              { label: "Pi", value: Math.PI },
              { label: "E", value: Math.E },
              { label: "Sqrt(2)", value: Math.SQRT2 },
              { label: "Ln(2)", value: Math.LN2 },
              { label: "Ln(10)", value: Math.LN10 },
              { label: "Log10(E)", value: Math.LOG10E }
            ] as quickValue}
              <button
                class="btn btn-sm bg-surface-100 dark:bg-surface-600 hover:bg-surface-200 dark:hover:bg-surface-500 
                  text-surface-700 dark:text-surface-200 rounded px-2 py-1 text-xs"
                onclick={() => { 
                  floatValue = formatNumber(quickValue.value); 
                }}
              >
                {quickValue.label}
              </button>
            {/each}
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
                About Float Constants
              </h3>
              <div class="text-xs text-blue-700 dark:text-blue-300 space-y-1">
                <p>• 64-bit double-precision floating-point numbers (IEEE 754)</p>
                <p>• Used for decimal calculations, scientific notation, and mathematical constants</p>
                <p>• Finite values only - HashLink VM doesn't support special IEEE 754 values</p>
                <p>• Range: ±4.9e-324 to ±1.8e308 (finite values only)</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Validation Summary -->
        <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
          <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Float Status</h3>
          <div class="space-y-1 text-xs">
            <div class={`flex items-center gap-2 ${isValidFloat ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {isValidFloat ? '✓' : '✗'}
              </span>
              Float value: {isValidFloat ? `Valid (${parsedValue})` : 'Required - must be a valid floating-point number'}
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