<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { OPCODE_OPTIONS } from "./opcode_options_full";
  import { invoke } from "@tauri-apps/api/core";
  import VirtualList from 'svelte-tiny-virtual-list';
  import ViewDocOpcodes from "./ViewDocOpcodes.svelte";

  // Use $props() instead of export let in Svelte 5
  let { modalMode = $bindable("create"), editFunctionIndex = $bindable(null) }: {
    modalMode: "edit" | "create";
    editFunctionIndex: number | null;
  } = $props();

  const dispatch = createEventDispatcher();

  // Svelte 5 state management
  let name = $state("");
  let index = $state("");
  let typeIdx = $state("");
  let registers = $state([0]);
  let opcodes = $state<any[]>([]);

  // Available data arrays
  let availableIntNames = $state<{ idx: number, value: string }[]>([]);
  let availableFloatNames = $state<{ idx: number, value: string }[]>([]);
  let availableFunctionNames = $state<{ idx: number, value: string }[]>([]);
  let availableTypes = $state<{ idx: number, name: string }[]>([]);
  let availableFnNames = $state<{ idx: number, value: string }[]>([]);

  // UI State
  let showOpcodeModal = $state(false);
  let editingOpcode = $state<{
    idx: number;
    key: string;
    params: Record<string, string>;
    query: string;
    insertPosition?: number; // For insert operations
  } | null>(null);
  let lastEditIndex = $state<number | null>(null);
  let showOpcodeDocumentation = $state(false);
  let selectedOpcodes = $state<Set<number>>(new Set());

  // Popover states - using a more centralized approach
  let activePopover = $state<string | null>(null);
  let popoverSearchQueries = $state<Record<string, string>>({});

  // Load function data when editing
  $effect(() => {
    if (editFunctionIndex !== lastEditIndex) {
      lastEditIndex = editFunctionIndex;
      
      if (editFunctionIndex !== null) {
        resetForm();
        fetchFunctionToEdit(editFunctionIndex);
      } else {
        resetForm();
      }
    }
  });

  interface FEdit {
    name: number;
    t: number;
    findex: number;
    ops: any[];
    regs: number[];
  }

  async function fetchFunctionToEdit(idx: number) {
    try {
      const fun = await invoke("get_function_full_info", { index: idx }) as FEdit;
      
      name = String(fun.name);
      typeIdx = String(fun.t);
      index = String(fun.findex);
      registers = fun.regs.map((t: number) => t);
      opcodes = fun.ops;
    } catch (e) {
      console.error("Failed to fetch function:", e);
      resetForm();
    }
  }

  function openOpcodeDocumentation() {
    showOpcodeDocumentation = true;
  }

  // Fetch all required data
  async function fetchAllData() {
    await Promise.all([
      fetchTypes(),
      fetchFnNames(), 
      fetchIntNames(),
      fetchFloatNames(),
      fetchFunctionNames()
    ]);
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

  async function fetchFnNames() {
    try {
      const rawStrings: string[] = await invoke("get_string_list");
      availableFnNames = rawStrings.map((s) => {
        const m = s.match(/^(.*)@(\d+)$/);
        if (m) return { value: m[1].trim(), idx: Number(m[2]) };
        return null;
      }).filter(Boolean) as { value: string; idx: number }[];
    } catch (e) {
      availableFnNames = [];
    }
  }

  async function fetchIntNames() {
    try {
      const rawInts: string[] = await invoke("get_int_list");
      availableIntNames = rawInts.map(s => {
        const m = s.match(/^(.*)@(\d+)$/);
        if (m) return { value: m[1].trim(), idx: Number(m[2]) };
        return null;
      }).filter(Boolean) as { value: string; idx: number }[];
    } catch (e) {
      availableIntNames = [];
    }
  }

  async function fetchFloatNames() {
    try {
      const rawFloats: string[] = await invoke("get_float_list");
      availableFloatNames = rawFloats.map(s => {
        const m = s.match(/^(.*)@(\d+)$/);
        if (m) return { value: m[1].trim(), idx: Number(m[2]) };
        return null;
      }).filter(Boolean) as { value: string; idx: number }[];
    } catch (e) {
      availableFloatNames = [];
    }
  }

  async function fetchFunctionNames() {
    try {
      const rawFuncs: string[] = await invoke("get_function_list");
      availableFunctionNames = rawFuncs.map(s => {
        const m = s.match(/^(.*)@(\d+)$/);
        if (m) return { value: m[1].trim(), idx: Number(m[2]) };
        return null;
      }).filter(Boolean) as { value: string; idx: number }[];
    } catch (e) {
      availableFunctionNames = [];
    }
  }

  // Register management
  function addRegister() {
    registers = [...registers, 0];
  }

  function removeRegister(idx: number) {
    if (registers.length <= 1) return;
    registers = registers.filter((_, i) => i !== idx);
  }

  function setRegister(idx: number, typeId: number) {
    registers = registers.map((v, i) => i === idx ? typeId : v);
    closePopover();
  }

  // Opcode management
  function openAddOpcode(insertPosition?: number) {
    editingOpcode = {
      idx: -1,
      key: "",
      params: {},
      query: "",
      insertPosition
    };
    showOpcodeModal = true;
  }

  function openEditOpcode(idx: number) {
    const op = opcodes[idx];
    const opkey = Object.keys(op)[0];
    editingOpcode = {
      idx,
      key: opkey,
      params: { ...op[opkey] },
      query: ""
    };
    showOpcodeModal = true;
  }

  function insertOpcode(position: number, opcode: any) {
    opcodes = [
      ...opcodes.slice(0, position),
      opcode,
      ...opcodes.slice(position)
    ];
  }

  function duplicateOpcode(idx: number) {
    const op = opcodes[idx];
    insertOpcode(idx + 1, JSON.parse(JSON.stringify(op)));
  }

  function moveOpcode(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    
    const newOpcodes = [...opcodes];
    const [movedItem] = newOpcodes.splice(fromIndex, 1);
    newOpcodes.splice(toIndex, 0, movedItem);
    opcodes = newOpcodes;
  }

  function confirmOpcodeEdit() {
    if (!editingOpcode) return;
    const key = editingOpcode.key;
    if (!key) return;

    const paramsSpec = OPCODE_OPTIONS.find(o => o.key === key)?.params || [];
    let params: Record<string, any> = {};
    
    for (let p of paramsSpec) {
      const paramKey = p.key;
      const v = editingOpcode.params[paramKey];
      const paramType = ('type' in p ? p.type : undefined);
      
      // Skip empty parameters - they're optional
      if (v === undefined || v === "") continue;
      
      if (paramType === "bool") {
        params[paramKey] = v === "true";
      } else if (paramType === "reg" || paramType === "int" || paramType === "function" || !paramType) {
        if (isNaN(+v)) continue; // Skip invalid numbers instead of failing
        params[paramKey] = Number(v);
      } else {
        params[paramKey] = v;
      }
    }

    const newOpcode = { [key]: params };
    
    if (editingOpcode.idx === -1) {
      // New opcode
      if (editingOpcode.insertPosition !== undefined) {
        insertOpcode(editingOpcode.insertPosition, newOpcode);
      } else {
        opcodes = [...opcodes, newOpcode];
      }
    } else {
      // Edit existing opcode
      opcodes = opcodes.map((item, i) => i === editingOpcode!.idx ? newOpcode : item);
    }
    
    closeOpcodeModal();
  }

  function setOpcodeParam(key: string, value: string) {
    if (!editingOpcode) return;
    editingOpcode = {
      ...editingOpcode,
      params: {
        ...editingOpcode.params,
        [key]: value
      }
    };
  }

  function setOpcodeType(key: string) {
    if (!editingOpcode) return;
    editingOpcode = {
      ...editingOpcode,
      key: key,
      params: {}
    };
  }

  function removeOpcode(idx: number) {
    opcodes = opcodes.filter((_, i) => i !== idx);
  }

  function closeOpcodeModal() {
    showOpcodeModal = false;
    editingOpcode = null;
  }

  // Selection and movement handlers
  function toggleSelection(index: number) {
    const newSelection = new Set(selectedOpcodes);
    if (newSelection.has(index)) {
      newSelection.delete(index);
    } else {
      newSelection.add(index);
    }
    selectedOpcodes = newSelection;
  }

  function selectAll() {
    selectedOpcodes = new Set(opcodes.map((_, i) => i));
  }

  function clearSelection() {
    selectedOpcodes = new Set();
  }

  function moveOpcodeUp(index: number) {
    if (index > 0) {
      moveOpcode(index, index - 1);
      // Update selection indices
      updateSelectionAfterMove(index, index - 1);
    }
  }

  function moveOpcodeDown(index: number) {
    if (index < opcodes.length - 1) {
      moveOpcode(index, index + 1);
      // Update selection indices
      updateSelectionAfterMove(index, index + 1);
    }
  }

  function updateSelectionAfterMove(fromIndex: number, toIndex: number) {
    const newSelection = new Set<number>();
    selectedOpcodes.forEach(idx => {
      if (idx === fromIndex) {
        newSelection.add(toIndex);
      } else if (fromIndex < toIndex && idx > fromIndex && idx <= toIndex) {
        newSelection.add(idx - 1);
      } else if (fromIndex > toIndex && idx < fromIndex && idx >= toIndex) {
        newSelection.add(idx + 1);
      } else {
        newSelection.add(idx);
      }
    });
    selectedOpcodes = newSelection;
  }

  // Bulk operations for selected opcodes
  function moveSelectedOpcodesUp() {
    const indices = Array.from(selectedOpcodes).sort((a, b) => a - b);
    if (indices.length === 0 || indices[0] === 0) return;
    
    for (const index of indices) {
      moveOpcode(index, index - 1);
    }
    
    // Update selection to follow moved opcodes
    const newSelection = new Set<number>();
    for (const index of indices) {
      newSelection.add(index - 1);
    }
    selectedOpcodes = newSelection;
  }

  function moveSelectedOpcodesDown() {
    const indices = Array.from(selectedOpcodes).sort((a, b) => b - a);
    if (indices.length === 0 || indices[0] === opcodes.length - 1) return;
    
    for (const index of indices) {
      moveOpcode(index, index + 1);
    }
    
    // Update selection to follow moved opcodes
    const newSelection = new Set<number>();
    for (const index of indices) {
      newSelection.add(index + 1);
    }
    selectedOpcodes = newSelection;
  }

  function removeSelectedOpcodes() {
    const indices = Array.from(selectedOpcodes).sort((a, b) => b - a);
    for (const index of indices) {
      opcodes = opcodes.filter((_, i) => i !== index);
    }
    selectedOpcodes = new Set();
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

  // Handle outside clicks
  function handleDocumentClick(e: MouseEvent) {
    const target = e.target as Element;
    if (!target.closest('.popover-container')) {
      closePopover();
    }
  }

  function handleEscape(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closePopover();
      if (showOpcodeModal) closeOpcodeModal();
      if (showOpcodeDocumentation) showOpcodeDocumentation = false;
    }
  }

  // Form submission
  function saveFunction() {
    if (modalMode === "edit" && editFunctionIndex !== null) {
      dispatch("edit", {
        functionIndex: editFunctionIndex,
        name,
        index: Number(index) || undefined,
        type: Number(typeIdx),
        registers,
        opcodes
      });
    } else {
      dispatch("save", {
        name,
        index: Number(index) || undefined,
        type: Number(typeIdx),
        registers,
        opcodes
      });
    }
    resetForm();
  }

  function resetForm() {
    name = "";
    index = "";
    typeIdx = "";
    registers = [0];
    opcodes = [];
  }

  // Validation functions
  function canSaveFunction() {
    return name && typeIdx && registers.length > 0;
  }

  // Filtered data for search
  $effect(() => {
    // This effect will re-run when search queries change
  });

  function getFilteredTypes(query: string) {
    return availableTypes.filter(t =>
      t.name.toLowerCase().includes(query.toLowerCase()) ||
      String(t.idx).includes(query)
    );
  }

  function getFilteredFnNames(query: string) {
    return availableFnNames.filter(t =>
      t.value.toLowerCase().includes(query.toLowerCase()) ||
      String(t.idx).includes(query)
    );
  }

  function getFilteredFunctions(query: string) {
    return availableFunctionNames.filter(t =>
      t.value.toLowerCase().includes(query.toLowerCase()) ||
      String(t.idx).includes(query)
    );
  }

  function getFilteredInts(query: string) {
    return availableIntNames.filter(t =>
      t.value.toLowerCase().includes(query.toLowerCase()) ||
      String(t.idx).includes(query)
    );
  }

  function getFilteredFloats(query: string) {
    return availableFloatNames.filter(t =>
      t.value.toLowerCase().includes(query.toLowerCase()) ||
      String(t.idx).includes(query)
    );
  }

  function getFilteredOpcodes(query: string) {
    return OPCODE_OPTIONS.filter(o =>
      o.label.toLowerCase().includes(query.toLowerCase()) ||
      o.key.toLowerCase().includes(query.toLowerCase())
    );
  }

  onMount(() => {
    fetchAllData();
    document.addEventListener("click", handleDocumentClick);
    document.addEventListener("keydown", handleEscape);
    
    return () => {
      document.removeEventListener("click", handleDocumentClick);
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
            {modalMode === "edit" ? "Edit Function" : "Create Function"}
          </h1>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            {modalMode === "edit" ? "Modify the selected function" : "Define a new function for the bytecode"}
          </p>
        </div>
        
        <!-- Save Button -->
        <button
          class="btn bg-success-500 hover:bg-success-600 text-white rounded-lg px-6 py-2 text-sm font-medium
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={saveFunction}
          disabled={!canSaveFunction()}
        >
          {modalMode === "edit" ? "Save Function" : "Create Function"}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-4xl mx-auto space-y-8">
        
        <!-- Basic Information Section -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="mb-6">
            <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
              Basic Information
            </h2>
            <p class="text-sm text-surface-600 dark:text-surface-400">
              Set the fundamental properties of your function. All fields are required except function index.
            </p>
          </div>
          
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- Function Name -->
            <div class="space-y-2">
              <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="function-name-select">
                Function Name *
              </label>
              <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
                Select from existing string constants in the bytecode
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
                  <span class={name ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                    {name ? availableFnNames.find(t => t.idx === +name)?.value ?? name : 'Select function name...'}
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
                        placeholder="Search names..."
                        bind:value={popoverSearchQueries.functionName}
                      />
                    </div>
                    <div class="max-h-60 overflow-y-auto">
                      {#each getFilteredFnNames(popoverSearchQueries.functionName || '') as fn}
                        <button
                          class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                            text-surface-900 dark:text-surface-100 flex items-center justify-between"
                          onclick={() => { name = String(fn.idx); closePopover(); }}
                        >
                          <span class="font-mono">{fn.value}</span>
                          <span class="text-xs text-surface-500">@{fn.idx}</span>
                        </button>
                      {/each}
                    </div>
                  </div>
                {/if}
              </div>
            </div>

            <!-- Function Index -->
            <div class="space-y-2">
              <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="function-index-input">
                Function Index
              </label>
              <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
                Leave empty to auto-assign the next available index
              </p>
              <input
                id="function-index-input"
                class="input w-full bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                  rounded-lg px-3 py-2 text-sm text-surface-900 dark:text-surface-100
                  focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                type="number"
                min="0"
                bind:value={index}
                placeholder="Auto-assigned if empty"
              />
            </div>

            <!-- Function Type -->
            <div class="space-y-2 md:col-span-2">
              <label class="block text-sm font-medium text-surface-700 dark:text-surface-300" for="function-type-select">
                Function Type *
              </label>
              <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
                Must be a function signature type (Type::Fun or Type::Method)
              </p>
              <div class="relative popover-container">
                <button
                  type="button"
                  id="function-type-select"
                  class="input w-full text-left flex items-center justify-between
                    bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                    rounded-lg px-3 py-2 text-sm hover:border-primary-400 focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                  onclick={() => openPopover('functionType')}
                >
                  <span class={typeIdx ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                    {typeIdx ? availableTypes.find(t => t.idx === +typeIdx)?.name ?? typeIdx : 'Select function type...'}
                  </span>
                  <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                  </svg>
                </button>
                
                {#if activePopover === 'functionType'}
                  <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                    <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                      <input
                        class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                          bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                          focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                        type="text"
                        placeholder="Search types..."
                        bind:value={popoverSearchQueries.functionType}
                      />
                    </div>
                    <div class="max-h-60 overflow-y-auto">
                      {#each getFilteredTypes(popoverSearchQueries.functionType || '') as type}
                        <button
                          class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                            text-surface-900 dark:text-surface-100 flex items-center justify-between"
                          onclick={() => { typeIdx = String(type.idx); closePopover(); }}
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
        </div>

        <!-- Registers Section -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
                Function Registers
              </h2>
              <p class="text-sm text-surface-600 dark:text-surface-400">
                Define the local variables/registers used by this function. At least one register is required.
              </p>
            </div>
            <button
              class="btn btn-sm bg-primary-500 hover:bg-primary-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
              onclick={addRegister}
            >
              Add Register
            </button>
          </div>
          
          <div class="space-y-3">
            {#each registers as reg, idx}
              <div class="flex items-center gap-3 p-3 bg-surface-50 dark:bg-surface-700 rounded-lg border border-surface-200 dark:border-surface-600">
                <span class="text-sm font-medium text-surface-600 dark:text-surface-400 min-w-[80px]">
                  Register {idx}
                </span>
                
                <div class="flex-1 relative popover-container">
                  <button
                    type="button"
                    class="input w-full text-left flex items-center justify-between
                      bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                      rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                    onclick={() => openPopover(`register-${idx}`)}
                  >
                    <span class="text-surface-900 dark:text-surface-100">
                      {availableTypes.find(t => t.idx === reg)?.name ?? reg}
                    </span>
                    <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                    </svg>
                  </button>
                  
                  {#if activePopover === `register-${idx}`}
                    <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                      <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                        <input
                          class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                            bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                            focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                          type="text"
                          placeholder="Search types..."
                          bind:value={popoverSearchQueries[`register-${idx}`]}
                        />
                      </div>
                      <div class="max-h-48 overflow-y-auto">
                        {#each getFilteredTypes(popoverSearchQueries[`register-${idx}`] || '') as type}
                          <button
                            class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                              text-surface-900 dark:text-surface-100 flex items-center justify-between"
                            onclick={() => setRegister(idx, type.idx)}
                          >
                            <span class="font-mono">{type.name}</span>
                            <span class="text-xs text-surface-500">@{type.idx}</span>
                          </button>
                        {/each}
                      </div>
                    </div>
                  {/if}
                </div>
                
                <button
                  class="btn btn-sm bg-error-500 hover:bg-error-600 text-white rounded-lg px-2 py-1.5 disabled:opacity-50"
                  onclick={() => removeRegister(idx)}
                  disabled={registers.length <= 1}
                  title={registers.length <= 1 ? "At least one register is required" : "Remove this register"}
                >
                  Remove
                </button>
              </div>
            {/each}
            
            {#if registers.length === 0}
              <div class="text-center py-8 text-surface-500 dark:text-surface-400">
                <p>No registers defined. Add at least one register to continue.</p>
              </div>
            {/if}
          </div>
        </div>

        <!-- Opcodes Section -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="flex items-center justify-between mb-6">
            <div>
              <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
                Function Opcodes
              </h2>
            </div>
            <div class="flex gap-2">
              <button
                class="btn btn-sm bg-blue-500 hover:bg-blue-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                onclick={() => {
                  console.log("📖 Docs button clicked!");
                  openOpcodeDocumentation();
                }}
                title="View opcode documentation"
              >
                📖 Docs
              </button>
              {#if selectedOpcodes.size > 0}
                <button
                  class="btn btn-sm bg-green-500 hover:bg-green-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                  onclick={moveSelectedOpcodesUp}
                  disabled={Array.from(selectedOpcodes).some(idx => idx === 0)}
                  title="Move selected opcodes up"
                >
                  ↑ Move Up
                </button>
                <button
                  class="btn btn-sm bg-green-500 hover:bg-green-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                  onclick={moveSelectedOpcodesDown}
                  disabled={Array.from(selectedOpcodes).some(idx => idx === opcodes.length - 1)}
                  title="Move selected opcodes down"
                >
                  ↓ Move Down
                </button>
                <button
                  class="btn btn-sm bg-red-500 hover:bg-red-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                  onclick={removeSelectedOpcodes}
                  title="Remove selected opcodes"
                >
                  🗑 Delete ({selectedOpcodes.size})
                </button>
                <button
                  class="btn btn-sm bg-orange-500 hover:bg-orange-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                  onclick={clearSelection}
                  title="Clear selection"
                >
                  Clear ({selectedOpcodes.size})
                </button>
              {/if}
              {#if opcodes.length > 0}
                <button
                  class="btn btn-sm bg-purple-500 hover:bg-purple-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                  onclick={selectAll}
                  title="Select all opcodes"
                >
                  Select All
                </button>
              {/if}
            </div>
          </div>
          
          <div class="space-y-1 max-h-96 overflow-y-auto">
            {#if opcodes.length === 0}
              <div class="text-center py-8 text-surface-500 dark:text-surface-400">
                <p class="mb-2">No opcodes defined.</p>
                <p class="text-xs mb-4">Add opcodes to implement the function logic. Functions can be empty but won't do anything useful.</p>
                <button
                  class="btn btn-sm bg-primary-500 hover:bg-primary-600 text-white rounded-lg px-4 py-2 text-sm font-medium"
                  onclick={() => openAddOpcode(0)}
                >
                  Add First Opcode
                </button>
              </div>
            {:else}
              {#each opcodes as op, idx}
                {@const isSelected = selectedOpcodes.has(idx)}
                
                <div class="group relative">
                  <div 
                    class="flex items-center gap-3 p-3 rounded-lg border transition-all
                      {isSelected 
                        ? 'bg-primary-50 dark:bg-primary-900/20 border-primary-300 dark:border-primary-600 ring-2 ring-primary-200 dark:ring-primary-800' 
                        : 'bg-surface-50 dark:bg-surface-700 border-surface-200 dark:border-surface-600 hover:bg-surface-100 dark:hover:bg-surface-600'}"
                  >
                    <!-- Selection checkbox -->
                    <div class="flex items-center">
                      <input
                        type="checkbox"
                        checked={isSelected}
                        onchange={() => toggleSelection(idx)}
                        class="w-4 h-4 text-primary-600 bg-surface-100 border-surface-300 rounded focus:ring-primary-500 dark:focus:ring-primary-600 dark:ring-offset-surface-800 focus:ring-2 dark:bg-surface-700 dark:border-surface-600"
                      />
                    </div>
                    
                    <!-- Movement arrows -->
                    <div class="flex flex-col gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        class="btn btn-xs bg-surface-300 hover:bg-surface-400 dark:bg-surface-600 dark:hover:bg-surface-500 
                          text-surface-700 dark:text-surface-200 rounded px-1 py-0.5 text-xs leading-none disabled:opacity-30"
                        onclick={(e) => { e.stopPropagation(); moveOpcodeUp(idx); }}
                        disabled={idx === 0}
                        title="Move up"
                      >
                        ↑
                      </button>
                      <button
                        class="btn btn-xs bg-surface-300 hover:bg-surface-400 dark:bg-surface-600 dark:hover:bg-surface-500 
                          text-surface-700 dark:text-surface-200 rounded px-1 py-0.5 text-xs leading-none disabled:opacity-30"
                        onclick={(e) => { e.stopPropagation(); moveOpcodeDown(idx); }}
                        disabled={idx === opcodes.length - 1}
                        title="Move down"
                      >
                        ↓
                      </button>
                    </div>
                    
                    <!-- Opcode content - clickable for selection -->
                    <div class="flex-1 flex items-center gap-3 cursor-pointer" onclick={() => toggleSelection(idx)}>
                      <!-- Index -->
                      <span class="text-sm font-medium text-surface-600 dark:text-surface-400 min-w-[40px]">
                        {idx}:
                      </span>
                      
                      <!-- Opcode name -->
                      <span class="font-mono text-sm font-bold text-primary-600 dark:text-primary-400">
                        {Object.keys(op)[0]}
                      </span>
                      
                      <!-- Parameters -->
                      <div class="flex-1 text-xs text-surface-600 dark:text-surface-400">
                        {#each Object.entries(op[Object.keys(op)[0]] ?? {}) as [k, v]}
                          <span class="inline-block mr-3">{k}: {v}</span>
                        {/each}
                      </div>
                    </div>
                    
                    <!-- Action buttons -->
                    <div class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      <button
                        class="btn btn-xs bg-green-500 hover:bg-green-600 text-white rounded px-2 py-1 text-xs"
                        onclick={(e) => { e.stopPropagation(); openAddOpcode(idx + 1); }}
                        title="Insert opcode after this one"
                      >
                        + Insert
                      </button>
                      <button
                        class="btn btn-xs bg-blue-500 hover:bg-blue-600 text-white rounded px-2 py-1 text-xs"
                        onclick={(e) => { e.stopPropagation(); duplicateOpcode(idx); }}
                        title="Duplicate this opcode"
                      >
                        Copy
                      </button>
                      <button
                        class="btn btn-xs bg-surface-200 hover:bg-surface-300 dark:bg-surface-600 dark:hover:bg-surface-500 
                          text-surface-700 dark:text-surface-200 rounded px-2 py-1 text-xs"
                        onclick={(e) => { e.stopPropagation(); openEditOpcode(idx); }}
                        title="Edit this opcode"
                      >
                        Edit
                      </button>
                      <button
                        class="btn btn-xs bg-error-500 hover:bg-error-600 text-white rounded px-2 py-1 text-xs"
                        onclick={(e) => { e.stopPropagation(); removeOpcode(idx); }}
                        title="Remove this opcode"
                      >
                        Remove
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>

        

        <!-- Validation Summary -->
        <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
          <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Function Status</h3>
          <div class="space-y-1 text-xs">
            <div class={`flex items-center gap-2 ${name ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {name ? '✓' : '✗'}
              </span>
              Function name: {name ? 'Selected' : 'Required'}
            </div>
            <div class={`flex items-center gap-2 ${typeIdx ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {typeIdx ? '✓' : '✗'}
              </span>
              Function type: {typeIdx ? 'Selected' : 'Required'}
            </div>
            <div class={`flex items-center gap-2 ${registers.length > 0 ? 'text-success-600' : 'text-error-600'}`}>
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                {registers.length > 0 ? '✓' : '✗'}
              </span>
              Registers: {registers.length > 0 ? `${registers.length} defined` : 'At least one required'}
            </div>
            <div class="flex items-center gap-2 text-surface-600 dark:text-surface-400">
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                ℹ
              </span>
              Opcodes: {opcodes.length} defined (optional)
            </div>
            <div class="flex items-center gap-2 text-surface-600 dark:text-surface-400">
              <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                ℹ
              </span>
              Function index: {index || 'Will be auto-assigned'}
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Opcode Modal (unchanged) -->
{#if showOpcodeModal && editingOpcode}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
    <div class="bg-white dark:bg-surface-800 rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-hidden">
      <div class="p-6 border-b border-surface-200 dark:border-surface-700">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-semibold text-surface-900 dark:text-surface-100">
            {editingOpcode.idx === -1 ? 
              (editingOpcode.insertPosition !== undefined ? 
                `Insert Opcode at Position ${editingOpcode.insertPosition}` : 
                "Add Opcode") : 
              "Edit Opcode"}
          </h3>
          <button
            class="btn btn-sm bg-surface-200 hover:bg-surface-300 dark:bg-surface-600 dark:hover:bg-surface-500 
              text-surface-700 dark:text-surface-200 rounded-lg px-3 py-1.5"
            onclick={closeOpcodeModal}
          >
            ×
          </button>
        </div>
      </div>
      
      <div class="p-6 overflow-y-auto max-h-[calc(90vh-120px)]">
        <!-- Opcode Selection -->
        <div class="space-y-4">
          <div>
            <label class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-2" for="opcode-search-input">
              Opcode Type
            </label>
            <input
              id="opcode-search-input"
              class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
              type="text"
              placeholder="Search opcodes..."
              value={editingOpcode?.query || ''}
              oninput={(e) => {
                const target = e.target as HTMLInputElement;
                if (target && editingOpcode) {
                  editingOpcode = { ...editingOpcode, query: target.value };
                }
              }}
            />
            
            <div class="mt-2 max-h-48 overflow-y-auto border border-surface-200 dark:border-surface-600 rounded-lg">
              {#each getFilteredOpcodes(editingOpcode?.query || '') as op}
                <button
                  class="w-full text-left px-3 py-2 hover:bg-surface-100 dark:hover:bg-surface-700 
                    text-surface-900 dark:text-surface-100 flex items-center gap-3
                    {editingOpcode?.key === op.key ? 'bg-primary-100 dark:bg-primary-900' : ''}"
                  onclick={() => setOpcodeType(op.key)}
                >
                  <span class="font-mono text-sm font-medium">{op.key}</span>
                  <span class="text-sm text-surface-600 dark:text-surface-400">{op.label}</span>
                </button>
              {/each}
            </div>
          </div>

          <!-- Parameters -->
          {#if editingOpcode?.key}
            {@const opcodeSpec = OPCODE_OPTIONS.find(o => o.key === editingOpcode!.key)}
            {#if opcodeSpec?.params && opcodeSpec.params.length > 0}
              <div class="space-y-4">
                <h4 class="text-sm font-medium text-surface-700 dark:text-surface-300">Parameters</h4>
                
                {#each opcodeSpec.params as param}
                  {@const paramKey = param.key}
                  <div>
                    <label class="block text-sm font-medium text-surface-700 dark:text-surface-300 mb-1" for="param-{paramKey}">
                      {paramKey}
                      {'type' in param ? `(${param.type})` : ''}
                    </label>
                    
                    {#if 'type' in param}
                      {#if param.type === 'reg'}
                        <!-- Register selector -->
                        <div class="relative popover-container">
                          <button
                            type="button"
                            id="param-{paramKey}"
                            class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                              bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100 text-left flex items-center justify-between"
                            onclick={() => openPopover(`opcode-reg-${paramKey}`)}
                          >
                            <span class={editingOpcode?.params[paramKey] ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500'}>
                              {editingOpcode?.params[paramKey] !== undefined ? 
                                `Register ${editingOpcode!.params[paramKey]}` :
                                'Select register...'}
                            </span>
                            <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                          </button>
                          
                          {#if activePopover === `opcode-reg-${paramKey}`}
                            <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                              <div class="max-h-48 overflow-y-auto">
                                {#each registers as _, rIdx}
                                  <button
                                    class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                      text-surface-900 dark:text-surface-100"
                                    onclick={() => { setOpcodeParam(paramKey, String(rIdx)); closePopover(); }}
                                  >
                                    Register {rIdx}
                                  </button>
                                {/each}
                              </div>
                            </div>
                          {/if}
                        </div>
                      
                      {:else if param.type === 'function'}
                        <!-- Function selector -->
                        <div class="relative popover-container">
                          <button
                            type="button"
                            class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                              bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100 text-left flex items-center justify-between"
                            onclick={() => openPopover(`opcode-function-${paramKey}`)}
                          >
                            <span class={editingOpcode?.params[paramKey] ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500'}>
                              {editingOpcode?.params[paramKey] ? 
                                availableFunctionNames.find(f => f.idx === +(editingOpcode!.params[paramKey] || 0))?.value ?? editingOpcode!.params[paramKey] :
                                'Select function...'}
                            </span>
                            <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                          </button>
                          
                          {#if activePopover === `opcode-function-${paramKey}`}
                            <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                              <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                                <input
                                  class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                    bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                  type="text"
                                  placeholder="Search functions..."
                                  bind:value={popoverSearchQueries[`opcode-function-${paramKey}`]}
                                />
                              </div>
                              <div class="max-h-48 overflow-y-auto">
                                {#each getFilteredFunctions(popoverSearchQueries[`opcode-function-${paramKey}`] || '') as fn}
                                  <button
                                    class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                      text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                    onclick={() => { setOpcodeParam(paramKey, String(fn.idx)); closePopover(); }}
                                  >
                                    <span class="font-mono">{fn.value}</span>
                                    <span class="text-xs text-surface-500">@{fn.idx}</span>
                                  </button>
                                {/each}
                              </div>
                            </div>
                          {/if}
                        </div>
                      
                      {:else if param.type === 'string'}
                        <!-- String selector -->
                        <div class="relative popover-container">
                          <button
                            type="button"
                            class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                              bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100 text-left flex items-center justify-between"
                            onclick={() => openPopover(`opcode-string-${paramKey}`)}
                          >
                            <span class={editingOpcode?.params[paramKey] ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500'}>
                              {editingOpcode?.params[paramKey] ? 
                                availableFnNames.find(s => s.idx === +(editingOpcode!.params[paramKey] || 0))?.value ?? editingOpcode!.params[paramKey] :
                                'Select string...'}
                            </span>
                            <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                          </button>
                          
                          {#if activePopover === `opcode-string-${paramKey}`}
                            <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                              <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                                <input
                                  class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                    bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                  type="text"
                                  placeholder="Search strings..."
                                  bind:value={popoverSearchQueries[`opcode-string-${paramKey}`]}
                                />
                              </div>
                              <div class="max-h-48 overflow-y-auto">
                                {#each getFilteredFnNames(popoverSearchQueries[`opcode-string-${paramKey}`] || '') as str}
                                  <button
                                    class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                      text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                    onclick={() => { setOpcodeParam(paramKey, String(str.idx)); closePopover(); }}
                                  >
                                    <span class="font-mono">{str.value}</span>
                                    <span class="text-xs text-surface-500">@{str.idx}</span>
                                  </button>
                                {/each}
                              </div>
                            </div>
                          {/if}
                        </div>
                      
                      {:else if param.type === 'int'}
                        <!-- Int selector -->
                        <div class="relative popover-container">
                          <button
                            type="button"
                            class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                              bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100 text-left flex items-center justify-between"
                            onclick={() => openPopover(`opcode-int-${paramKey}`)}
                          >
                            <span class={editingOpcode?.params[paramKey] ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500'}>
                              {editingOpcode?.params[paramKey] ? 
                                availableIntNames.find(i => i.idx === +(editingOpcode!.params[paramKey] || 0))?.value ?? editingOpcode!.params[paramKey] :
                                'Select int...'}
                            </span>
                            <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                          </button>
                          
                          {#if activePopover === `opcode-int-${paramKey}`}
                            <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                              <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                                <input
                                  class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                    bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                  type="text"
                                  placeholder="Search ints..."
                                  bind:value={popoverSearchQueries[`opcode-int-${paramKey}`]}
                                />
                              </div>
                              <div class="max-h-48 overflow-y-auto">
                                {#each getFilteredInts(popoverSearchQueries[`opcode-int-${paramKey}`] || '') as int}
                                  <button
                                    class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                      text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                    onclick={() => { setOpcodeParam(paramKey, String(int.idx)); closePopover(); }}
                                  >
                                    <span class="font-mono">{int.value}</span>
                                    <span class="text-xs text-surface-500">@{int.idx}</span>
                                  </button>
                                {/each}
                              </div>
                            </div>
                          {/if}
                        </div>
                      
                      {:else if param.type === 'float'}
                        <!-- Float selector -->
                        <div class="relative popover-container">
                          <button
                            type="button"
                            class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                              bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100 text-left flex items-center justify-between"
                            onclick={() => openPopover(`opcode-float-${paramKey}`)}
                          >
                            <span class={editingOpcode?.params[paramKey] ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500'}>
                              {editingOpcode?.params[paramKey] ? 
                                availableFloatNames.find(f => f.idx === +(editingOpcode!.params[paramKey] || 0))?.value ?? editingOpcode!.params[paramKey] :
                                'Select float...'}
                            </span>
                            <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                          </button>
                          
                          {#if activePopover === `opcode-float-${paramKey}`}
                            <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                              <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                                <input
                                  class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                    bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                  type="text"
                                  placeholder="Search floats..."
                                  bind:value={popoverSearchQueries[`opcode-float-${paramKey}`]}
                                />
                              </div>
                              <div class="max-h-48 overflow-y-auto">
                                {#each getFilteredFloats(popoverSearchQueries[`opcode-float-${paramKey}`] || '') as float}
                                  <button
                                    class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                      text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                    onclick={() => { setOpcodeParam(paramKey, String(float.idx)); closePopover(); }}
                                  >
                                    <span class="font-mono">{float.value}</span>
                                    <span class="text-xs text-surface-500">@{float.idx}</span>
                                  </button>
                                {/each}
                              </div>
                            </div>
                          {/if}
                        </div>
                      
                      {:else if param.type === 'type'}
                        <!-- Type selector -->
                        <div class="relative popover-container">
                          <button
                            type="button"
                            class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                              bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100 text-left flex items-center justify-between"
                            onclick={() => openPopover(`opcode-type-${paramKey}`)}
                          >
                            <span class={editingOpcode?.params[paramKey] ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500'}>
                              {editingOpcode?.params[paramKey] ? 
                                availableTypes.find(t => t.idx === +(editingOpcode!.params[paramKey] || 0))?.name ?? editingOpcode!.params[paramKey] :
                                'Select type...'}
                            </span>
                            <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                            </svg>
                          </button>
                          
                          {#if activePopover === `opcode-type-${paramKey}`}
                            <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                              <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                                <input
                                  class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                    bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                  type="text"
                                  placeholder="Search types..."
                                  bind:value={popoverSearchQueries[`opcode-type-${paramKey}`]}
                                />
                              </div>
                              <div class="max-h-48 overflow-y-auto">
                                {#each getFilteredTypes(popoverSearchQueries[`opcode-type-${paramKey}`] || '') as type}
                                  <button
                                    class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                      text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                    onclick={() => { setOpcodeParam(paramKey, String(type.idx)); closePopover(); }}
                                  >
                                    <span class="font-mono">{type.name}</span>
                                    <span class="text-xs text-surface-500">@{type.idx}</span>
                                  </button>
                                {/each}
                              </div>
                            </div>
                          {/if}
                        </div>
                      
                      {:else if param.type === 'bool'}
                        <!-- Boolean selector -->
                        <select
                          class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                            bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                          value={editingOpcode?.params[paramKey] || ''}
                          onchange={(e) => {
                            const target = e.target as HTMLSelectElement;
                            if (target) setOpcodeParam(paramKey, target.value);
                          }}
                        >
                          <option value="">Select boolean...</option>
                          <option value="true">true</option>
                          <option value="false">false</option>
                        </select>
                      
                      {:else}
                        <!-- Default number input -->
                        <input
                          class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                            bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                          type="number"
                          value={editingOpcode?.params[paramKey] || ''}
                          oninput={(e) => {
                            const target = e.target as HTMLInputElement;
                            if (target) setOpcodeParam(paramKey, target.value);
                          }}
                        />
                      {/if}
                    {:else}
                      <!-- Default number input for parameters without type -->
                      <input
                        class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                          bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                        type="number"
                        value={editingOpcode?.params[paramKey] || ''}
                        oninput={(e) => {
                          const target = e.target as HTMLInputElement;
                          if (target) setOpcodeParam(paramKey, target.value);
                        }}
                      />
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          {/if}
        </div>
      </div>
      
      <div class="p-6 border-t border-surface-200 dark:border-surface-700">
        <div class="flex justify-end gap-3">
          <button
            class="btn bg-surface-200 hover:bg-surface-300 dark:bg-surface-600 dark:hover:bg-surface-500 
              text-surface-700 dark:text-surface-200 rounded-lg px-4 py-2 text-sm font-medium"
            onclick={closeOpcodeModal}
          >
            Cancel
          </button>
          <button
            class="btn bg-primary-500 hover:bg-primary-600 text-white rounded-lg px-4 py-2 text-sm font-medium"
            onclick={confirmOpcodeEdit}
          >
            {editingOpcode?.idx === -1 ? 
              (editingOpcode?.insertPosition !== undefined ? "Insert Opcode" : "Add Opcode") : 
              "Update Opcode"}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<ViewDocOpcodes bind:isOpen={showOpcodeDocumentation} />