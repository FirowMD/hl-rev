<script lang="ts">
  import { onMount, createEventDispatcher } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { typesRefreshKey } from "./types";

  // Use $props() instead of export let in Svelte 5
  let { modalMode = $bindable("create"), editTypeIndex = $bindable(null) }: {
    modalMode: "edit" | "create";
    editTypeIndex: number | null;
  } = $props();

  const dispatch = createEventDispatcher();

  // Type kind categories
  const TYPE_KINDS = {
    primitive: {
      label: "Primitive Types",
      options: [
        { key: "void", label: "Void", description: "No value" },
        { key: "ui8", label: "UI8", description: "8-bit unsigned integer" },
        { key: "ui16", label: "UI16", description: "16-bit unsigned integer" },
        { key: "i32", label: "I32", description: "32-bit signed integer" },
        { key: "i64", label: "I64", description: "64-bit signed integer" },
        { key: "f32", label: "F32", description: "32-bit float" },
        { key: "f64", label: "F64", description: "64-bit float" },
        { key: "bool", label: "Bool", description: "Boolean value" },
        { key: "bytes", label: "Bytes", description: "Byte array" },
        { key: "dyn", label: "Dynamic", description: "Dynamic type" },
        { key: "array", label: "Array", description: "Array type" },
        { key: "type", label: "Type", description: "Type object" },
        { key: "dynobj", label: "DynObj", description: "Dynamic object" }
      ]
    },
    composite: {
      label: "Composite Types",  
      options: [
        { key: "obj", label: "Object", description: "Class definition" },
        { key: "struct", label: "Struct", description: "Struct definition" },
        { key: "enum", label: "Enum", description: "Enumeration type" },
        { key: "virtual", label: "Virtual", description: "Anonymous object" },
        { key: "abstract", label: "Abstract", description: "Abstract class" }
      ]
    },
    function: {
      label: "Function Types",
      options: [
        { key: "fun", label: "Function", description: "Function signature" },
        { key: "method", label: "Method", description: "Method signature" }
      ]
    },
    wrapper: {
      label: "Wrapper Types",
      options: [
        { key: "ref", label: "Reference", description: "Reference to another type" },
        { key: "null", label: "Nullable", description: "Nullable wrapper" },
        { key: "packed", label: "Packed", description: "Packed wrapper" }
      ]
    }
  };

  // Form state
  let typeKind = $state("");
  let typeName = $state("");
  let superType = $state("");
  let globalRef = $state("");
  let innerType = $state(""); // For wrapper types
  
  // Function type fields
  let functionArgs = $state<string[]>([]);
  let functionReturn = $state("");
  
  // Object/Struct fields
  let objectFields = $state<{name: string, fieldType: string}[]>([]);
  let objectProtos = $state<{name: string, findex: string, pindex: string}[]>([]);
  
  // Enum constructs
  let enumConstructs = $state<{name: string, params: string[]}[]>([]);

  // Available reference data
  let availableTypes = $state<{ idx: number, name: string }[]>([]);
  let availableStrings = $state<{ idx: number, value: string }[]>([]);
  let availableFunctions = $state<{ idx: number, value: string }[]>([]);
  let availableGlobals = $state<{ idx: number, value: string }[]>([]);

  // UI State
  let activePopover = $state<string | null>(null);
  let popoverSearchQueries = $state<Record<string, string>>({});
  let lastEditIndex = $state<number | null>(null);

  // Load type data when editing
  $effect(() => {
    if (editTypeIndex !== lastEditIndex) {
      lastEditIndex = editTypeIndex;
      
      if (editTypeIndex !== null) {
        resetForm();
        fetchTypeToEdit(editTypeIndex);
      } else {
        resetForm();
      }
    }
  });

  async function fetchTypeToEdit(idx: number) {
    try {
      const type = await invoke("get_type_full_info", { index: idx });
      console.log("Loaded type for editing:", type);
      
      parseTypeForEditing(type);
      
    } catch (e) {
      console.error("Failed to fetch type:", e);
      resetForm();
    }
  }

  function parseTypeForEditing(type: any) {
    resetForm();
    
    // Parse different type variants for editing
    if (typeof type === 'string') {
      typeKind = type;
    } else if (type.Void !== undefined) {
      typeKind = "void";
    } else if (type.UI8 !== undefined) {
      typeKind = "ui8";
    } else if (type.UI16 !== undefined) {
      typeKind = "ui16";
    } else if (type.I32 !== undefined) {
      typeKind = "i32";
    } else if (type.I64 !== undefined) {
      typeKind = "i64";
    } else if (type.F32 !== undefined) {
      typeKind = "f32";
    } else if (type.F64 !== undefined) {
      typeKind = "f64";
    } else if (type.Bool !== undefined) {
      typeKind = "bool";
    } else if (type.Bytes !== undefined) {
      typeKind = "bytes";
    } else if (type.Dyn !== undefined) {
      typeKind = "dyn";
    } else if (type.Array !== undefined) {
      typeKind = "array";
    } else if (type.Type !== undefined) {
      typeKind = "type";
    } else if (type.DynObj !== undefined) {
      typeKind = "dynobj";
    } else if (type.Ref !== undefined) {
      typeKind = "ref";
      innerType = String(type.Ref);
    } else if (type.Null !== undefined) {
      typeKind = "null";
      innerType = String(type.Null);
    } else if (type.Packed !== undefined) {
      typeKind = "packed";
      innerType = String(type.Packed);
    } else if (type.Fun !== undefined) {
      typeKind = "fun";
      functionArgs = type.Fun.args.map(arg => String(arg));
      functionReturn = String(type.Fun.ret);
    } else if (type.Method !== undefined) {
      typeKind = "method";
      functionArgs = type.Method.args.map(arg => String(arg));
      functionReturn = String(type.Method.ret);
    } else if (type.Obj !== undefined) {
      typeKind = "obj";
      typeName = String(type.Obj.name);
      superType = type.Obj.super_ ? String(type.Obj.super_) : "";
      globalRef = type.Obj.global ? String(type.Obj.global) : "";
      objectFields = type.Obj.own_fields.map(field => ({
        name: String(field.name),
        fieldType: String(field.t)
      }));
      objectProtos = type.Obj.protos.map(proto => ({
        name: String(proto.name),
        findex: String(proto.findex),
        pindex: String(proto.pindex)
      }));
    } else if (type.Struct !== undefined) {
      typeKind = "struct";
      typeName = String(type.Struct.name);
      superType = type.Struct.super_ ? String(type.Struct.super_) : "";
      globalRef = type.Struct.global ? String(type.Struct.global) : "";
      objectFields = type.Struct.own_fields.map(field => ({
        name: String(field.name),
        fieldType: String(field.t)
      }));
      objectProtos = type.Struct.protos.map(proto => ({
        name: String(proto.name),
        findex: String(proto.findex),
        pindex: String(proto.pindex)
      }));
    } else if (type.Enum !== undefined) {
      typeKind = "enum";
      typeName = String(type.Enum.name);
      globalRef = type.Enum.global ? String(type.Enum.global) : "";
      enumConstructs = type.Enum.constructs.map(construct => ({
        name: String(construct.name),
        params: construct.params.map(p => String(p))
      }));
    } else if (type.Abstract !== undefined) {
      typeKind = "abstract";
      typeName = String(type.Abstract.name);
    } else if (type.Virtual !== undefined) {
      typeKind = "virtual";
      objectFields = type.Virtual.fields.map(field => ({
        name: String(field.name),
        fieldType: String(field.t)
      }));
    }
  }

  // Fetch all required reference data
  async function fetchAllData() {
    await Promise.all([
      fetchTypes(),
      fetchStrings(),
      fetchFunctions(),
      fetchGlobals()
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

  async function fetchFunctions() {
    try {
      const rawFunctions: string[] = await invoke("get_function_list");
      availableFunctions = rawFunctions.map(s => {
        const m = s.match(/^(.*)@(\d+)$/);
        if (m) return { value: m[1].trim(), idx: Number(m[2]) };
        return null;
      }).filter(Boolean) as { value: string; idx: number }[];
    } catch (e) {
      availableFunctions = [];
    }
  }

  async function fetchGlobals() {
    try {
      const rawGlobals: string[] = await invoke("get_global_list");
      availableGlobals = rawGlobals.map(s => {
        const m = s.match(/^(.*)@(\d+)$/);
        if (m) return { value: m[1].trim(), idx: Number(m[2]) };
        return null;
      }).filter(Boolean) as { value: string; idx: number }[];
    } catch (e) {
      availableGlobals = [];
    }
  }

  // Field management for objects/structs
  function addObjectField() {
    objectFields = [...objectFields, { name: "", fieldType: "" }];
  }

  function removeObjectField(idx: number) {
    objectFields = objectFields.filter((_, i) => i !== idx);
  }

  function addObjectProto() {
    objectProtos = [...objectProtos, { name: "", findex: "", pindex: "0" }];
  }

  function removeObjectProto(idx: number) {
    objectProtos = objectProtos.filter((_, i) => i !== idx);
  }

  // Function args management
  function addFunctionArg() {
    functionArgs = [...functionArgs, ""];
  }

  function removeFunctionArg(idx: number) {
    functionArgs = functionArgs.filter((_, i) => i !== idx);
  }

  // Enum construct management
  function addEnumConstruct() {
    enumConstructs = [...enumConstructs, { name: "", params: [] }];
  }

  function removeEnumConstruct(idx: number) {
    enumConstructs = enumConstructs.filter((_, i) => i !== idx);
  }

  function addEnumParam(constructIdx: number) {
    enumConstructs = enumConstructs.map((construct, i) => 
      i === constructIdx ? 
        { ...construct, params: [...construct.params, ""] } : 
        construct
    );
  }

  function removeEnumParam(constructIdx: number, paramIdx: number) {
    enumConstructs = enumConstructs.map((construct, i) => 
      i === constructIdx ? 
        { ...construct, params: construct.params.filter((_, j) => j !== paramIdx) } : 
        construct
    );
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

  function getFilteredStrings(query: string) {
    return availableStrings.filter(s =>
      s.value.toLowerCase().includes(query.toLowerCase()) ||
      String(s.idx).includes(query)
    );
  }

  function getFilteredFunctions(query: string) {
    return availableFunctions.filter(f =>
      f.value.toLowerCase().includes(query.toLowerCase()) ||
      String(f.idx).includes(query)
    );
  }

  function getFilteredGlobals(query: string) {
    return availableGlobals.filter(g =>
      g.value.toLowerCase().includes(query.toLowerCase()) ||
      String(g.idx).includes(query)
    );
  }

  // Form submission
  async function saveType() {
    const typeData = {
      type_kind: typeKind,
      name: typeName || null,
      super_type: superType ? Number(superType) : null,
      global: globalRef ? Number(globalRef) : null,
      inner_type: innerType ? Number(innerType) : null,
      args: functionArgs.length > 0 ? functionArgs.map(arg => Number(arg)).filter(n => !isNaN(n)) : null,
      ret: functionReturn ? Number(functionReturn) : null,
      fields: objectFields.length > 0 ? objectFields.map(f => ({
        name: f.name,
        field_type: Number(f.fieldType)
      })).filter(f => f.name && !isNaN(f.field_type)) : null,
      protos: objectProtos.length > 0 ? objectProtos.map(p => ({
        name: p.name,
        findex: Number(p.findex),
        pindex: Number(p.pindex) || 0
      })).filter(p => p.name && !isNaN(p.findex)) : null,
      constructs: enumConstructs.length > 0 ? enumConstructs.map(c => ({
        name: c.name,
        params: c.params.map(p => Number(p)).filter(n => !isNaN(n))
      })).filter(c => c.name) : null
    };

    try {
      if (modalMode === "edit" && editTypeIndex !== null) {
        await invoke("update_type", { index: editTypeIndex, input: typeData });
        dispatch("edit", {
          typeIndex: editTypeIndex,
          ...typeData
        });
      } else {
        await invoke("create_type", { input: typeData });
        dispatch("save", typeData);
      }
      
      // Trigger refresh of types list
      typesRefreshKey.update(n => n + 1);
      
      resetForm();
    } catch (error) {
      console.error("Failed to save type:", error);
      alert(`Failed to save type: ${error}`);
    }
  }

  function resetForm() {
    typeKind = "";
    typeName = "";
    superType = "";
    globalRef = "";
    innerType = "";
    functionArgs = [];
    functionReturn = "";
    objectFields = [];
    objectProtos = [];
    enumConstructs = [];
  }

  // Validation
  function canSaveType() {
    if (!typeKind) return false;
    
    // Type-specific validation
    if (["obj", "struct", "enum", "abstract"].includes(typeKind)) {
      if (!typeName) return false;
    }
    
    if (["fun", "method"].includes(typeKind)) {
      if (!functionReturn) return false;
    }
    
    if (["ref", "null", "packed"].includes(typeKind)) {
      if (!innerType) return false;
    }
    
    return true;
  }

  // Get current type kind info
  let currentTypeKind = $derived(Object.values(TYPE_KINDS)
    .flatMap(category => category.options)
    .find(option => option.key === typeKind));

  // Handle escape key
  function handleEscape(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closePopover();
    }
  }

  onMount(() => {
    fetchAllData();
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
            {modalMode === "edit" ? "Edit Type" : "Create Type"}
          </h1>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            {modalMode === "edit" ? "Modify the selected type definition" : "Define a new type for the bytecode"}
          </p>
        </div>
        
        <!-- Save Button -->
        <button
          class="btn bg-success-500 hover:bg-success-600 text-white rounded-lg px-6 py-2 text-sm font-medium
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={saveType}
          disabled={!canSaveType()}
        >
          {modalMode === "edit" ? "Save Type" : "Create Type"}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto p-6">
      <div class="max-w-4xl mx-auto space-y-8">
        
        <!-- Type Kind Selection -->
        <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
          <div class="mb-6">
            <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-2">
              Type Kind
            </h2>
            <p class="text-sm text-surface-600 dark:text-surface-400">
              Select the kind of type you want to create. This determines what fields are available.
            </p>
          </div>
          
          <!-- Type Kind Categories -->
          <div class="space-y-6">
            {#each Object.entries(TYPE_KINDS) as [categoryKey, category]}
              <div>
                <h3 class="text-md font-medium text-surface-800 dark:text-surface-200 mb-3">
                  {category.label}
                </h3>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                  {#each category.options as option}
                    <button
                      class="p-3 text-left rounded-lg border transition-colors
                        {typeKind === option.key 
                          ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20' 
                          : 'border-surface-200 dark:border-surface-600 hover:border-primary-300 hover:bg-surface-50 dark:hover:bg-surface-700'}"
                      onclick={() => { typeKind = option.key; }}
                    >
                      <div class="font-medium text-surface-900 dark:text-surface-100 text-sm">
                        {option.label}
                      </div>
                      <div class="text-xs text-surface-600 dark:text-surface-400 mt-1">
                        {option.description}
                      </div>
                    </button>
                  {/each}
                </div>
              </div>
            {/each}
          </div>

          <!-- Selected Type Info -->
          {#if currentTypeKind}
            <div class="mt-6 p-4 bg-primary-50 dark:bg-primary-900/20 rounded-lg border border-primary-200 dark:border-primary-700">
              <div class="flex items-center gap-2">
                <span class="text-primary-600 dark:text-primary-400 font-medium">Selected:</span>
                <span class="font-mono text-primary-800 dark:text-primary-300">{currentTypeKind.label}</span>
                <span class="text-primary-600 dark:text-primary-400">-</span>
                <span class="text-primary-700 dark:text-primary-300">{currentTypeKind.description}</span>
              </div>
            </div>
          {/if}
        </div>

        <!-- Type-Specific Configuration -->
        {#if typeKind}
          <!-- Named Types (obj, struct, enum, abstract) -->
          {#if ["obj", "struct", "enum", "abstract"].includes(typeKind)}
            <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
              <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-4">
                {typeKind.charAt(0).toUpperCase() + typeKind.slice(1)} Configuration
              </h2>
              
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <!-- Type Name -->
                <div class="space-y-2">
                  <label class="block text-sm font-medium text-surface-700 dark:text-surface-300">
                    Type Name *
                  </label>
                  <div class="relative popover-container">
                    <button
                      type="button"
                      class="input w-full text-left flex items-center justify-between
                        bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                        rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                      onclick={() => openPopover('typeName')}
                    >
                      <span class={typeName ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                        {typeName ? availableStrings.find(s => s.idx === +typeName)?.value ?? typeName : 'Select type name...'}
                      </span>
                      <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                      </svg>
                    </button>
                    
                    {#if activePopover === 'typeName'}
                      <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                        <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                          <input
                            class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                              bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                              focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
                            type="text"
                            placeholder="Search strings..."
                            bind:value={popoverSearchQueries.typeName}
                          />
                        </div>
                        <div class="max-h-60 overflow-y-auto">
                          {#each getFilteredStrings(popoverSearchQueries.typeName || '') as str}
                            <button
                              class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                text-surface-900 dark:text-surface-100 flex items-center justify-between"
                              onclick={() => { typeName = String(str.idx); closePopover(); }}
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

                <!-- Global Reference (for obj, struct, enum) -->
                {#if ["obj", "struct", "enum"].includes(typeKind)}
                  <div class="space-y-2">
                    <label class="block text-sm font-medium text-surface-700 dark:text-surface-300">
                      Global Reference
                    </label>
                    <div class="relative popover-container">
                      <button
                        type="button"
                        class="input w-full text-left flex items-center justify-between
                          bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                          rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                        onclick={() => openPopover('globalRef')}
                      >
                        <span class={globalRef ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                          {globalRef ? availableGlobals.find(g => g.idx === +globalRef)?.value ?? globalRef : 'Select global...'}
                        </span>
                        <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                        </svg>
                      </button>
                      
                      {#if activePopover === 'globalRef'}
                        <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                          <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                            <input
                              class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                              type="text"
                              placeholder="Search globals..."
                              bind:value={popoverSearchQueries.globalRef}
                            />
                          </div>
                          <div class="max-h-60 overflow-y-auto">
                            {#each getFilteredGlobals(popoverSearchQueries.globalRef || '') as global}
                              <button
                                class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                  text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                onclick={() => { globalRef = String(global.idx); closePopover(); }}
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
                {/if}

                <!-- Super Type (for obj, struct) -->
                {#if ["obj", "struct"].includes(typeKind)}
                  <div class="space-y-2 md:col-span-2">
                    <label class="block text-sm font-medium text-surface-700 dark:text-surface-300">
                      Super Type (Inheritance)
                    </label>
                    <div class="relative popover-container">
                      <button
                        type="button"
                        class="input w-full text-left flex items-center justify-between
                          bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                          rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                        onclick={() => openPopover('superType')}
                      >
                        <span class={superType ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                          {superType ? availableTypes.find(t => t.idx === +superType)?.name ?? superType : 'No inheritance...'}
                        </span>
                        <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                        </svg>
                      </button>
                      
                      {#if activePopover === 'superType'}
                        <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                          <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                            <input
                              class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                              type="text"
                              placeholder="Search types..."
                              bind:value={popoverSearchQueries.superType}
                            />
                          </div>
                          <div class="max-h-60 overflow-y-auto">
                            <button
                              class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                text-surface-900 dark:text-surface-100"
                              onclick={() => { superType = ""; closePopover(); }}
                            >
                              <span class="italic text-surface-500">No inheritance</span>
                            </button>
                            {#each getFilteredTypes(popoverSearchQueries.superType || '') as type}
                              <button
                                class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                  text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                onclick={() => { superType = String(type.idx); closePopover(); }}
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
                {/if}
              </div>

              <!-- Object/Struct Fields -->
              {#if ["obj", "struct", "virtual"].includes(typeKind)}
                <div class="space-y-4 mt-6">
                  <div class="flex items-center justify-between">
                    <h3 class="text-md font-medium text-surface-800 dark:text-surface-200">Fields</h3>
                    <button
                      class="btn btn-sm bg-primary-500 hover:bg-primary-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                      onclick={addObjectField}
                    >
                      Add Field
                    </button>
                  </div>
                  
                  {#each objectFields as field, idx}
                    <div class="flex items-center gap-3 p-3 bg-surface-50 dark:bg-surface-700 rounded-lg">
                      <span class="text-sm font-medium text-surface-600 dark:text-surface-400 min-w-[60px]">
                        Field {idx}:
                      </span>
                      
                      <div class="flex-1 relative popover-container">
                        <button
                          type="button"
                          class="input w-full text-left flex items-center justify-between
                            bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                            rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                          onclick={() => openPopover(`fieldName-${idx}`)}
                        >
                          <span class={field.name ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                            {field.name ? availableStrings.find(s => s.idx === +field.name)?.value ?? field.name : 'Select name...'}
                          </span>
                          <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                          </svg>
                        </button>
                        
                        {#if activePopover === `fieldName-${idx}`}
                          <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                            <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                              <input
                                class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                  bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                type="text"
                                placeholder="Search strings..."
                                bind:value={popoverSearchQueries[`fieldName-${idx}`]}
                              />
                            </div>
                            <div class="max-h-48 overflow-y-auto">
                              {#each getFilteredStrings(popoverSearchQueries[`fieldName-${idx}`] || '') as str}
                                <button
                                  class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                    text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                  onclick={() => { 
                                    objectFields = objectFields.map((f, i) => i === idx ? { ...f, name: String(str.idx) } : f);
                                    closePopover(); 
                                  }}
                                >
                                  <span class="font-mono">{str.value}</span>
                                  <span class="text-xs text-surface-500">@{str.idx}</span>
                                </button>
                              {/each}
                            </div>
                          </div>
                        {/if}
                      </div>

                      <div class="flex-1 relative popover-container">
                        <button
                          type="button"
                          class="input w-full text-left flex items-center justify-between
                            bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                            rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                          onclick={() => openPopover(`fieldType-${idx}`)}
                        >
                          <span class={field.fieldType ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                            {field.fieldType ? availableTypes.find(t => t.idx === +field.fieldType)?.name ?? field.fieldType : 'Select type...'}
                          </span>
                          <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                          </svg>
                        </button>
                        
                        {#if activePopover === `fieldType-${idx}`}
                          <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                            <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                              <input
                                class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                  bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                type="text"
                                placeholder="Search types..."
                                bind:value={popoverSearchQueries[`fieldType-${idx}`]}
                              />
                            </div>
                            <div class="max-h-48 overflow-y-auto">
                              {#each getFilteredTypes(popoverSearchQueries[`fieldType-${idx}`] || '') as type}
                                <button
                                  class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                    text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                  onclick={() => { 
                                    objectFields = objectFields.map((f, i) => i === idx ? { ...f, fieldType: String(type.idx) } : f);
                                    closePopover(); 
                                  }}
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
                        class="btn btn-sm bg-error-500 hover:bg-error-600 text-white rounded-lg px-2 py-1.5"
                        onclick={() => removeObjectField(idx)}
                      >
                        Remove
                      </button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <!-- Function Types (fun, method) -->
          {#if ["fun", "method"].includes(typeKind)}
            <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
              <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-4">
                Function Signature
              </h2>
              
              <!-- Arguments -->
              <div class="space-y-4">
                <div class="flex items-center justify-between">
                  <h3 class="text-md font-medium text-surface-800 dark:text-surface-200">Arguments</h3>
                  <button
                    class="btn btn-sm bg-primary-500 hover:bg-primary-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                    onclick={addFunctionArg}
                  >
                    Add Argument
                  </button>
                </div>
                
                {#each functionArgs as arg, idx}
                  <div class="flex items-center gap-3 p-3 bg-surface-50 dark:bg-surface-700 rounded-lg">
                    <span class="text-sm font-medium text-surface-600 dark:text-surface-400 min-w-[60px]">
                      Arg {idx}:
                    </span>
                    
                    <div class="flex-1 relative popover-container">
                      <button
                        type="button"
                        class="input w-full text-left flex items-center justify-between
                          bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                          rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                        onclick={() => openPopover(`functionArg-${idx}`)}
                      >
                        <span class={arg ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                          {arg ? availableTypes.find(t => t.idx === +arg)?.name ?? arg : 'Select type...'}
                        </span>
                        <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                        </svg>
                      </button>
                      
                      {#if activePopover === `functionArg-${idx}`}
                        <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                          <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                            <input
                              class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                              type="text"
                              placeholder="Search types..."
                              bind:value={popoverSearchQueries[`functionArg-${idx}`]}
                            />
                          </div>
                          <div class="max-h-48 overflow-y-auto">
                            {#each getFilteredTypes(popoverSearchQueries[`functionArg-${idx}`] || '') as type}
                              <button
                                class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                  text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                onclick={() => { 
                                  functionArgs = functionArgs.map((a, i) => i === idx ? String(type.idx) : a);
                                  closePopover(); 
                                }}
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
                      class="btn btn-sm bg-error-500 hover:bg-error-600 text-white rounded-lg px-2 py-1.5"
                      onclick={() => removeFunctionArg(idx)}
                    >
                      Remove
                    </button>
                  </div>
                {/each}
              </div>

              <!-- Return Type -->
              <div class="space-y-2 mt-6">
                <label class="block text-sm font-medium text-surface-700 dark:text-surface-300">
                  Return Type *
                </label>
                <div class="relative popover-container">
                  <button
                    type="button"
                    class="input w-full text-left flex items-center justify-between
                      bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                      rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                    onclick={() => openPopover('functionReturn')}
                  >
                    <span class={functionReturn ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                      {functionReturn ? availableTypes.find(t => t.idx === +functionReturn)?.name ?? functionReturn : 'Select return type...'}
                    </span>
                    <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                    </svg>
                  </button>
                  
                  {#if activePopover === 'functionReturn'}
                    <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                      <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                        <input
                          class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                            bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                          type="text"
                          placeholder="Search types..."
                          bind:value={popoverSearchQueries.functionReturn}
                        />
                      </div>
                      <div class="max-h-60 overflow-y-auto">
                        {#each getFilteredTypes(popoverSearchQueries.functionReturn || '') as type}
                          <button
                            class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                              text-surface-900 dark:text-surface-100 flex items-center justify-between"
                            onclick={() => { functionReturn = String(type.idx); closePopover(); }}
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
          {/if}

          <!-- Wrapper Types (ref, null, packed) -->
          {#if ["ref", "null", "packed"].includes(typeKind)}
            <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
              <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-4">
                {typeKind.charAt(0).toUpperCase() + typeKind.slice(1)} Configuration
              </h2>
              
              <div class="space-y-2">
                <label class="block text-sm font-medium text-surface-700 dark:text-surface-300">
                  Inner Type *
                </label>
                <p class="text-xs text-surface-500 dark:text-surface-400 mb-2">
                  The type that this {typeKind} wraps
                </p>
                <div class="relative popover-container">
                  <button
                    type="button"
                    class="input w-full text-left flex items-center justify-between
                      bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                      rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                    onclick={() => openPopover('innerType')}
                  >
                    <span class={innerType ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                      {innerType ? availableTypes.find(t => t.idx === +innerType)?.name ?? innerType : 'Select inner type...'}
                    </span>
                    <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                    </svg>
                  </button>
                  
                  {#if activePopover === 'innerType'}
                    <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                      <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                        <input
                          class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                            bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                          type="text"
                          placeholder="Search types..."
                          bind:value={popoverSearchQueries.innerType}
                        />
                      </div>
                      <div class="max-h-60 overflow-y-auto">
                        {#each getFilteredTypes(popoverSearchQueries.innerType || '') as type}
                          <button
                            class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                              text-surface-900 dark:text-surface-100 flex items-center justify-between"
                            onclick={() => { innerType = String(type.idx); closePopover(); }}
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
          {/if}

          <!-- Enum Types -->
          {#if typeKind === "enum"}
            <div class="bg-white dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-6">
              <h2 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-4">
                Enum Variants
              </h2>
              
              <div class="space-y-4">
                <div class="flex items-center justify-between">
                  <h3 class="text-md font-medium text-surface-800 dark:text-surface-200">Variants</h3>
                  <button
                    class="btn btn-sm bg-primary-500 hover:bg-primary-600 text-white rounded-lg px-3 py-1.5 text-sm font-medium"
                    onclick={addEnumConstruct}
                  >
                    Add Variant
                  </button>
                </div>
                
                {#each enumConstructs as construct, idx}
                  <div class="p-4 bg-surface-50 dark:bg-surface-700 rounded-lg">
                    <div class="flex items-center gap-3 mb-3">
                      <span class="text-sm font-medium text-surface-600 dark:text-surface-400 min-w-[80px]">
                        Variant {idx}:
                      </span>
                      
                      <div class="flex-1 relative popover-container">
                        <button
                          type="button"
                          class="input w-full text-left flex items-center justify-between
                            bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                            rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                          onclick={() => openPopover(`enumName-${idx}`)}
                        >
                          <span class={construct.name ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                            {construct.name ? availableStrings.find(s => s.idx === +construct.name)?.value ?? construct.name : 'Select name...'}
                          </span>
                          <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                          </svg>
                        </button>
                        
                        {#if activePopover === `enumName-${idx}`}
                          <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                            <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                              <input
                                class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                  bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                type="text"
                                placeholder="Search strings..."
                                bind:value={popoverSearchQueries[`enumName-${idx}`]}
                              />
                            </div>
                            <div class="max-h-48 overflow-y-auto">
                              {#each getFilteredStrings(popoverSearchQueries[`enumName-${idx}`] || '') as str}
                                <button
                                  class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                    text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                  onclick={() => { 
                                    enumConstructs = enumConstructs.map((c, i) => i === idx ? { ...c, name: String(str.idx) } : c);
                                    closePopover(); 
                                  }}
                                >
                                  <span class="font-mono">{str.value}</span>
                                  <span class="text-xs text-surface-500">@{str.idx}</span>
                                </button>
                              {/each}
                            </div>
                          </div>
                        {/if}
                      </div>
                      
                      <button
                        class="btn btn-sm bg-error-500 hover:bg-error-600 text-white rounded-lg px-2 py-1.5"
                        onclick={() => removeEnumConstruct(idx)}
                      >
                        Remove
                      </button>
                    </div>

                    <!-- Enum Parameters -->
                    <div class="space-y-2">
                      <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-surface-700 dark:text-surface-300">Parameters</span>
                        <button
                          class="btn btn-sm bg-secondary-500 hover:bg-secondary-600 text-white rounded-lg px-2 py-1 text-xs"
                          onclick={() => addEnumParam(idx)}
                        >
                          Add Param
                        </button>
                      </div>
                      
                      {#each construct.params as param, paramIdx}
                        <div class="flex items-center gap-2">
                          <div class="flex-1 relative popover-container">
                            <button
                              type="button"
                              class="input w-full text-left flex items-center justify-between
                                bg-white dark:bg-surface-700 border border-surface-300 dark:border-surface-600
                                rounded-lg px-3 py-2 text-sm hover:border-primary-400"
                              onclick={() => openPopover(`enumParam-${idx}-${paramIdx}`)}
                            >
                              <span class={param ? 'text-surface-900 dark:text-surface-100' : 'text-surface-500 dark:text-surface-400'}>
                                {param ? availableTypes.find(t => t.idx === +param)?.name ?? param : 'Select type...'}
                              </span>
                              <svg class="w-4 h-4 text-surface-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                              </svg>
                            </button>
                            
                            {#if activePopover === `enumParam-${idx}-${paramIdx}`}
                              <div class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-surface-800 border border-surface-200 dark:border-surface-700 rounded-lg shadow-lg z-50">
                                <div class="p-3 border-b border-surface-200 dark:border-surface-700">
                                  <input
                                    class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-md 
                                      bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100"
                                    type="text"
                                    placeholder="Search types..."
                                    bind:value={popoverSearchQueries[`enumParam-${idx}-${paramIdx}`]}
                                  />
                                </div>
                                <div class="max-h-48 overflow-y-auto">
                                  {#each getFilteredTypes(popoverSearchQueries[`enumParam-${idx}-${paramIdx}`] || '') as type}
                                    <button
                                      class="w-full px-3 py-2 text-left text-sm hover:bg-surface-100 dark:hover:bg-surface-700 
                                        text-surface-900 dark:text-surface-100 flex items-center justify-between"
                                      onclick={() => { 
                                        enumConstructs = enumConstructs.map((c, i) => i === idx ? {
                                          ...c,
                                          params: c.params.map((p, j) => j === paramIdx ? String(type.idx) : p)
                                        } : c);
                                        closePopover(); 
                                      }}
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
                            class="btn btn-sm bg-error-500 hover:bg-error-600 text-white rounded-lg px-2 py-1"
                            onclick={() => removeEnumParam(idx, paramIdx)}
                          >
                            Remove
                          </button>
                        </div>
                      {/each}
                    </div>
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Validation Summary -->
          <div class="bg-surface-100 dark:bg-surface-800 rounded-lg border border-surface-200 dark:border-surface-700 p-4">
            <h3 class="text-sm font-medium text-surface-700 dark:text-surface-300 mb-2">Type Status</h3>
            <div class="space-y-1 text-xs">
              <div class={`flex items-center gap-2 ${typeKind ? 'text-success-600' : 'text-error-600'}`}>
                <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                  {typeKind ? '✓' : '✗'}
                </span>
                Type kind: {typeKind ? `Selected (${typeKind})` : 'Required'}
              </div>
              
              {#if ["obj", "struct", "enum", "abstract"].includes(typeKind)}
                <div class={`flex items-center gap-2 ${typeName ? 'text-success-600' : 'text-error-600'}`}>
                  <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                    {typeName ? '✓' : '✗'}
                  </span>
                  Type name: {typeName ? 'Selected' : 'Required for named types'}
                </div>
              {/if}
              
              {#if ["fun", "method"].includes(typeKind)}
                <div class={`flex items-center gap-2 ${functionReturn ? 'text-success-600' : 'text-error-600'}`}>
                  <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                    {functionReturn ? '✓' : '✗'}
                  </span>
                  Return type: {functionReturn ? 'Selected' : 'Required for function types'}
                </div>
              {/if}
              
              {#if ["ref", "null", "packed"].includes(typeKind)}
                <div class={`flex items-center gap-2 ${innerType ? 'text-success-600' : 'text-error-600'}`}>
                  <span class="w-4 h-4 rounded-full flex items-center justify-center text-white text-xs">
                    {innerType ? '✓' : '✗'}
                  </span>
                  Inner type: {innerType ? 'Selected' : 'Required for wrapper types'}
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>