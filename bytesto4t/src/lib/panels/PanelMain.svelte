<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { setContext } from 'svelte';
  import ViewDashboard from "../elems/ViewDashboard.svelte";
  import ViewInspector from "../elems/ViewInspector.svelte";
  import ViewDisassembler from "../elems/ViewDisassembler.svelte";
  import ViewDecompiler from "../elems/ViewDecompiler.svelte";
  import ViewTools from "../elems/ViewTools.svelte";
  import ViewSettings from "../elems/ViewSettings.svelte";
  import ViewAIDecompiler from "../elems/ViewAIDecompiler.svelte";
  import ViewConstructor from "../elems/ViewConstructor.svelte";
  import type { FileData, BytecodeItemSelectedEvent } from '../elems/types';
  import { functionsRefreshKey, functionToEdit, typeToEdit, globalToEdit, nativeToEdit, constantToEdit, stringToEdit, intToEdit, floatToEdit, mainPanelTab, stringsRefreshKey, intsRefreshKey, floatsRefreshKey } from "../elems/types";

  let fileData = $state<FileData | undefined>();

  // References to constructor component for external edit triggers
  let constructorRef = $state<ViewConstructor>();

  const isLargeFile = $derived((fileData?.size ?? 0) > 1024 * 1024);
  
  let hexScrollPosition = $state(0);
  let hexTargetOffset = $state(-1);
  let lastKnownFoffset = $state<number | null>(null);

  const tabs = [
    { id: 'dashboard', label: 'Dashboard', component: ViewDashboard },
    { id: 'inspector', label: 'Inspector', component: ViewInspector },
    { id: 'disassembler', label: 'Disassembler', component: ViewDisassembler },
    { id: 'decompiler', label: 'Decompiler', component: ViewDecompiler },
    { id: 'idecompiler', label: 'AI Decompiler', component: ViewAIDecompiler },
    { id: 'constructor', label: 'Constructor', component: ViewConstructor },
    { id: 'tools', label: 'Tools', component: ViewTools },
    { id: 'settings', label: 'Settings', component: ViewSettings }
  ];

  async function loadFile() {
    try {
      const path = await invoke('get_target_file_path') as string;
      const bytes = await invoke('read_binary_file', { path }) as number[];
      
      fileData = {
        buffer: new Uint8Array(bytes),
        size: bytes.length,
        name: path.split('/').pop() ?? 'unknown'
      };
    } catch (error) {
      console.error('Error loading file:', error);
      fileData = undefined;
    }
  }

  onMount(() => {
    setContext('tools', { elementIndex: null, references: [] });
    loadFile();
  });

  // Handle edit requests from external sources (e.g., from lists in other components)
  $effect(() => {
    if ($functionToEdit !== null) {
      mainPanelTab.set("constructor");
      constructorRef?.editFunction($functionToEdit);
      functionToEdit.set(null); // Reset after handling
    }
  });

  $effect(() => {
    if ($typeToEdit !== null) {
      mainPanelTab.set("constructor");
      constructorRef?.editType($typeToEdit);
      typeToEdit.set(null);
    }
  });

  $effect(() => {
    if ($globalToEdit !== null) {
      mainPanelTab.set("constructor");
      constructorRef?.editGlobal($globalToEdit);
      globalToEdit.set(null);
    }
  });

  $effect(() => {
    if ($nativeToEdit !== null) {
      mainPanelTab.set("constructor");
      constructorRef?.editNative($nativeToEdit);
      nativeToEdit.set(null);
    }
  });

  $effect(() => {
    if ($constantToEdit !== null) {
      mainPanelTab.set("constructor");
      constructorRef?.editConstant($constantToEdit);
      constantToEdit.set(null);
    }
  });

  $effect(() => {
    if ($stringToEdit !== null) {
      mainPanelTab.set("constructor");
      constructorRef?.editString($stringToEdit);
      stringToEdit.set(null);
    }
  });

  $effect(() => {
    if ($intToEdit !== null) {
      mainPanelTab.set("constructor");
      constructorRef?.editInt($intToEdit);
      intToEdit.set(null);
    }
  });

  $effect(() => {
    if ($floatToEdit !== null) {
      mainPanelTab.set("constructor");
      constructorRef?.editFloat($floatToEdit);
      floatToEdit.set(null);
    }
  });

  // Handle save events from constructor components
  async function handleFunctionSave(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail) return;
    try {
      await invoke("create_function", {
        input: {
          name: detail.name,
          t: detail.type,
          findex: detail.index || undefined,
          ops: detail.opcodes,
          regs: detail.registers,
        }
      });
      functionsRefreshKey.update(k => k + 1);
      console.log("Function created successfully");
    } catch (error) {
      console.error("Error creating function:", error);
      // TODO: Show user-friendly error message
    }
  }

  async function handleFunctionEdit(event: CustomEvent<any>) {
    const { functionIndex, ...detail } = event.detail;
    if (functionIndex == null) return;
    try {
      await invoke("update_function", {
        index: functionIndex,
        input: {
          name: String(detail.name),
          t: detail.type,
          findex: detail.index || undefined,
          ops: detail.opcodes,
          regs: detail.registers,
        }
      });
      functionsRefreshKey.update(k => k + 1);
      console.log("Function updated successfully");
    } catch (error) {
      console.error("Error editing function:", error);
      // TODO: Show user-friendly error message
    }
  }

  // Type CRUD handlers
  async function handleTypeSave(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail) return;
    try {
      await invoke("create_type", { input: detail });
      // Trigger refresh for any components that show types
      functionsRefreshKey.update(k => k + 1);
      console.log("Type created successfully");
    } catch (error) {
      console.error("Error creating type:", error);
      // TODO: Show user-friendly error message
    }
  }

  async function handleTypeEdit(event: CustomEvent<any>) {
    const { typeIndex, ...detail } = event.detail;
    if (typeIndex == null) return;
    try {
      await invoke("update_type", {
        index: typeIndex,
        input: detail
      });
      functionsRefreshKey.update(k => k + 1);
      console.log("Type updated successfully");
    } catch (error) {
      console.error("Error editing type:", error);
      // TODO: Show user-friendly error message
    }
  }

  // String CRUD handlers
  async function handleStringSave(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail) return;
    try {
      await invoke("create_string", { input: detail });
      stringsRefreshKey.update(k => k + 1);
      console.log("String created successfully");
    } catch (error) {
      console.error("Error creating string:", error);
      // TODO: Show user-friendly error message
    }
  }

  async function handleStringEdit(event: CustomEvent<any>) {
    const { stringIndex, ...detail } = event.detail;
    if (stringIndex == null) return;
    try {
      await invoke("update_string", {
        index: stringIndex,
        input: detail
      });
      stringsRefreshKey.update(k => k + 1);
      console.log("String updated successfully");
    } catch (error) {
      console.error("Error editing string:", error);
      // TODO: Show user-friendly error message
    }
  }

  // Int CRUD handlers
  async function handleIntSave(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail) return;
    try {
      await invoke("create_int", { input: detail });
      intsRefreshKey.update(k => k + 1);
      console.log("Int created successfully");
    } catch (error) {
      console.error("Error creating int:", error);
      // TODO: Show user-friendly error message
    }
  }

  async function handleIntEdit(event: CustomEvent<any>) {
    const { intIndex, ...detail } = event.detail;
    if (intIndex == null) return;
    try {
      await invoke("update_int", {
        index: intIndex,
        input: detail
      });
      intsRefreshKey.update(k => k + 1);
      console.log("Int updated successfully");
    } catch (error) {
      console.error("Error editing int:", error);
      // TODO: Show user-friendly error message
    }
  }

  // Float CRUD handlers
  async function handleFloatSave(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail) return;
    try {
      await invoke("create_float", { input: detail });
      floatsRefreshKey.update(k => k + 1);
      console.log("Float created successfully");
    } catch (error) {
      console.error("Error creating float:", error);
      // TODO: Show user-friendly error message
    }
}

async function handleFloatEdit(event: CustomEvent<any>) {
  const { floatIndex, ...detail } = event.detail;
  if (floatIndex == null) return;
  try {
    await invoke("update_float", {
      index: floatIndex,
      input: detail
    });
    floatsRefreshKey.update(k => k + 1);
    console.log("Float updated successfully");
  } catch (error) {
    console.error("Error editing float:", error);
    // TODO: Show user-friendly error message
  }
}

  // Global CRUD handlers
  async function handleGlobalSave(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail) return;
    try {
      await invoke("create_global", { input: detail });
      functionsRefreshKey.update(k => k + 1);
      console.log("Global created successfully");
    } catch (error) {
      console.error("Error creating global:", error);
      // TODO: Show user-friendly error message
    }
  }

  async function handleGlobalEdit(event: CustomEvent<any>) {
    const { globalIndex, ...detail } = event.detail;
    if (globalIndex == null) return;
    try {
      await invoke("update_global", {
        index: globalIndex,
        input: detail
      });
      functionsRefreshKey.update(k => k + 1);
      console.log("Global updated successfully");
    } catch (error) {
      console.error("Error editing global:", error);
      // TODO: Show user-friendly error message
    }
  }

  // Native CRUD handlers
  async function handleNativeSave(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail) return;
    try {
      await invoke("create_native", { input: detail });
      functionsRefreshKey.update(k => k + 1);
      console.log("Native created successfully");
    } catch (error) {
      console.error("Error creating native:", error);
      // TODO: Show user-friendly error message
    }
  }

  async function handleNativeEdit(event: CustomEvent<any>) {
    const { nativeIndex, ...detail } = event.detail;
    if (nativeIndex == null) return;
    try {
      await invoke("update_native", {
        index: nativeIndex,
        input: detail
      });
      functionsRefreshKey.update(k => k + 1);
      console.log("Native updated successfully");
    } catch (error) {
      console.error("Error editing native:", error);
      // TODO: Show user-friendly error message
    }
  }

  // Constant CRUD handlers
  async function handleConstantSave(event: CustomEvent<any>) {
    const detail = event.detail;
    if (!detail) return;
    try {
      await invoke("create_constant", { input: detail });
      functionsRefreshKey.update(k => k + 1);
      console.log("Constant created successfully");
    } catch (error) {
      console.error("Error creating constant:", error);
      // TODO: Show user-friendly error message
    }
  }

  async function handleConstantEdit(event: CustomEvent<any>) {
    const { constantIndex, ...detail } = event.detail;
    if (constantIndex == null) return;
    try {
      await invoke("update_constant", {
        index: constantIndex,
        input: detail
      });
      functionsRefreshKey.update(k => k + 1);
      console.log("Constant updated successfully");
    } catch (error) {
      console.error("Error editing constant:", error);
      // TODO: Show user-friendly error message
    }
  }
</script>

<div class="w-full h-full bg-surface-950">
  <div class="flex border-b border-surface-700 truncate overflow-x-auto">
    {#each tabs as tab}
      <button
        class="px-4 py-1 {$mainPanelTab === tab.id ? 'bg-surface-800 border-b-2 border-primary-500' : 'hover:bg-surface-800/50'}"
        onclick={() => mainPanelTab.set(tab.id)}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="px-2 py-1 h-[calc(100%-3rem)] overflow-hidden">
    {#each tabs as tab}
      {#if $mainPanelTab === tab.id}
        {#if tab.id === "constructor"}
          <ViewConstructor
            bind:this={constructorRef}
            on:functionSave={handleFunctionSave}
            on:functionEdit={handleFunctionEdit}
            on:typeSave={handleTypeSave}
            on:typeEdit={handleTypeEdit}
            on:globalSave={handleGlobalSave}
            on:globalEdit={handleGlobalEdit}
            on:nativeSave={handleNativeSave}
            on:nativeEdit={handleNativeEdit}
            on:constantSave={handleConstantSave}
            on:constantEdit={handleConstantEdit}
            on:stringSave={handleStringSave}
            on:stringEdit={handleStringEdit}
            on:intSave={handleIntSave}
            on:intEdit={handleIntEdit}
            on:floatSave={handleFloatSave}
            on:floatEdit={handleFloatEdit}
          />
        {:else}
          <svelte:component this={tab.component} />
        {/if}
      {/if}
    {/each}
  </div>
</div>