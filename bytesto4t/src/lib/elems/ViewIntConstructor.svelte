<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  // Use $props() instead of export let in Svelte 5
  let { modalMode = $bindable("create"), editIntIndex = $bindable(null) }: {
    modalMode: "edit" | "create";
    editIntIndex: number | null;
  } = $props();

  const dispatch = createEventDispatcher();

  // Form state
  let intValue = $state<number | string>("");

  // UI State
  let lastEditIndex = $state<number | null>(null);
  let inputMode = $state<"decimal" | "hex" | "binary">("decimal");

  // Load int data when editing
  $effect(() => {
    if (editIntIndex !== lastEditIndex) {
      lastEditIndex = editIntIndex;
      
      if (editIntIndex !== null) {
        resetForm();
        fetchIntToEdit(editIntIndex);
      } else {
        resetForm();
      }
    }
  });

  async function fetchIntToEdit(idx: number) {
    try {
      const value = await invoke("get_int_full_info", { index: idx }) as number;
      intValue = value;
    } catch (e) {
      console.error("Failed to fetch int:", e);
      resetForm();
    }
  }

  // Helper functions for number format conversion
  function formatNumber(value: number): string {
    switch (inputMode) {
      case "hex":
        return "0x" + value.toString(16).toUpperCase();
      case "binary":
        return "0b" + value.toString(2);
      default:
        return value.toString();
    }
  }

  function parseInput(input: string): number | null {
    if (typeof input === "number") return input;
    
    const trimmed = input.trim();
    if (trimmed === "") return null;
    
    try {
      if (trimmed.startsWith("0x") || trimmed.startsWith("0X")) {
        return parseInt(trimmed.slice(2), 16);
      } else if (trimmed.startsWith("0b") || trimmed.startsWith("0B")) {
        return parseInt(trimmed.slice(2), 2);
      } else {
        return parseInt(trimmed, 10);
      }
    } catch {
      return null;
    }
  }

  // Computed value for display
  let parsedValue = $derived(typeof intValue === "number" ? intValue : parseInput(intValue as string));
  let isValidInt = $derived(parsedValue !== null && Number.isInteger(parsedValue) && parsedValue >= -2147483648 && parsedValue <= 2147483647);

  // Form submission
  function saveInt() {
    const finalValue = typeof intValue === "number" ? intValue : parseInput(intValue as string);
    
    if (finalValue === null) return;

    const intData = {
      value: finalValue
    };

    if (modalMode === "edit" && editIntIndex !== null) {
      dispatch("edit", {
        intIndex: editIntIndex,
        ...intData
      });
    } else {
      dispatch("save", intData);
    }
    resetForm();
  }

  function resetForm() {
    intValue = "";
    inputMode = "decimal";
  }

  // Validation
  function canSaveInt() {
    return isValidInt;
  }

  // Handle input mode change
  function changeInputMode(mode: "decimal" | "hex" | "binary") {
    inputMode = mode;
    if (parsedValue !== null) {
      intValue = formatNumber(parsedValue);
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
            {modalMode === "edit" ? "Edit Integer" : "Create Integer"}
          </h1>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            {modalMode === "edit" ? "Modify the selected integer constant" : "Add a new integer constant to the bytecode"}
          </p>
        </div>
        
        <!-- Save Button -->
        <button
          class="btn bg-success-500 hover:bg-success-600 text-white rounded-lg px-6 py-2 text-sm font-medium
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={saveInt}
          disabled={!canSaveInt()}
        >
          {modalMode === "edit" ? "Save Integer" : "Create Integer"}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-2xl mx-auto space-y-8">
        
        <!-- Integer Configuration -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="mb-6">
            <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
              Integer Value Configuration
            </h2>
            <p class="text-sm text-surface-600 dark:text-surface-400">
              32-bit signed integer constants used for numeric literals and calculations.
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
                class="btn btn-sm {inputMode === 'hex' ? 'bg-primary-500 text-white' : 'bg-surface-200 dark:bg-surface-600'} 
                  hover:bg-primary-600 rounded-lg px-3 py-1.5 text-sm"
                onclick={() => changeInputMode('hex')}
              >
                Hexadecimal
              </button>
              <button
                class="btn btn-sm {inputMode === 'binary' ? 'bg-primary-500 text-white' : 'bg-surface-200 dark:bg-surface-600'} 
                  hover:bg-primary-600 rounded-lg px-3 py-1.5 text-sm"
                onclick={() => changeInputMode('binary')}
              >
                Binary
              </button>
            </div>
          </div>

          <!-- Integer Value Input -->
          <div class="space-y-2">
            <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="int-value-input">
              Integer Value *
            </label>
            <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
              Enter a 32-bit signed integer (-2,147,483,648 to 2,147,483,647)
              {#if inputMode === "hex"}• Use 0x prefix for hexadecimal (e.g., 0xFF, 0x1A2B){/if}
              {#if inputMode === "binary"}• Use 0b prefix for binary (e.g., 0b1010, 0b11110000){/if}
            </p>
            <input
              id="int-value-input"
              class="input w-full bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                rounded-lg px-3 py-2 text-sm text-surface-900 dark:text-surface-100
                focus:border-primary-500 focus:ring-1 focus:ring-primary-500 font-mono
                {!isValidInt && intValue !== '' ? 'border-error-500' : ''}"
              type="text"
              bind:value={intValue}
              placeholder={inputMode === "hex" ? "0xFF" : inputMode === "binary" ? "0b1010" : "42"}
            />
            
            {#if parsedValue !== null && isValidInt}
              <div class="mt-2 p-2 bg-success-50 dark:bg-success-900/20 rounded border border-success-200 dark:border-success-700">
                <div class="text-xs text-success-700 dark:text-success-300 space-y-1">
                  <div><strong>Decimal:</strong> {parsedValue}</div>
                  <div><strong>Hexadecimal:</strong> 0x{parsedValue.toString(16).toUpperCase()}</div>
                  <div><strong>Binary:</strong> 0b{parsedValue.toString(2)}</div>
                  <div><strong>Range check:</strong> ✓ Valid 32-bit signed integer</div>
                </div>
              </div>
            {:else if intValue !== "" && !isValidInt}
              <div class="mt-2 p-2 bg-error-50 dark:bg-error-900/20 rounded border border-error-200 dark:border-error-700">
                <div class="text-xs text-error-700 dark:text-error-300">
                  {#if parsedValue === null}
                    Invalid number format
                  {:else if !Number.isInteger(parsedValue)}
                    Must be a whole number
                  {:else}
                    Value must be between -2,147,483,648 and 2,147,483,647
                  {/if}
                </div>
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
                About Integer Constants
              </h3>
              <div class="text-xs text-blue-700 dark:text-blue-300 space-y-1">
                <p>• 32-bit signed integers stored in the constant pool</p>
                <p>• Used for numeric literals, array indices, and calculations</p>
                <p>• Supports decimal, hexadecimal (0x), and binary (0b) input formats</p>
                <p>• Range: -2,147,483,648 to 2,147,483,647</p>
              </div>
            </div>
          </div>
        </div>

        <!-- Validation Summary -->
        <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
          <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Integer Status</h3>
          <div class="space-y-1 text-xs">
            <div class={`flex items-center gap-2 ${isValidInt ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {isValidInt ? '✓' : '✗'}
              </span>
              Integer value: {isValidInt ? `Valid (${parsedValue})` : 'Required - must be a valid 32-bit integer'}
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