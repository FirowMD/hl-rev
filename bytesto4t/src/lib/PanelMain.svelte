<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { setContext } from 'svelte';
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import ViewDashboard from "./ViewDashboard.svelte";
  import ViewInspector from "./ViewInspector.svelte";
  import ViewDisassembler from "./ViewDisassembler.svelte";
  import ViewDecompiler from "./ViewDecompiler.svelte";
  import ViewTools from "./ViewTools.svelte";
  import ViewSettings from "./ViewSettings.svelte";
  import ViewAIDecompiler from "./ViewAIDecompiler.svelte";

  let fileData = $state<{
    buffer: Uint8Array,
    size: number,
    name: string
  } | undefined>();

  const isLargeFile = $derived((fileData?.size ?? 0) > 1024 * 1024);
  
  let hexScrollPosition = $state(0);
  let hexTargetOffset = $state(-1);
  let lastKnownFoffset = $state<number | null>(null);

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

  function handleHexScroll(event: CustomEvent<{ position: number }>) {
    hexScrollPosition = event.detail.position;
  }

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const foffset = await invoke("get_selected_item_foffset") as string;
      if (foffset) {
        const offset = parseInt(foffset);
        if (!isNaN(offset)) {
          lastKnownFoffset = offset;
          // Only update target offset if we're already on hex tab
          if (tabSet === 4) {
            hexTargetOffset = offset;
          }
        }
      }
    } catch (error) {
      console.error('Error getting foffset:', error);
    }
  }

  // Watch for tab changes
  $effect(() => {
    if (tabSet === 4 && lastKnownFoffset !== null) {
      hexTargetOffset = lastKnownFoffset;
    }
  });

  onMount(() => {
    setContext('tools', { elementIndex: null, references: [] });
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    loadFile();
  });

  onDestroy(() => {
    window.removeEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
  });

  let tabSet = $state(0);
</script>

<div class="flex flex-col h-full p-2">
  <TabGroup class="h-full" regionPanel="h-[calc(100%-3rem)]" regionList="h-10">
    <Tab bind:group={tabSet} name="tabDashboard" value={0}>Dashboard</Tab>
    <Tab bind:group={tabSet} name="tabInspector" value={1}>Inspector</Tab>
    <Tab bind:group={tabSet} name="tabDisassembler" value={2}>Disassembler</Tab>
    <Tab bind:group={tabSet} name="tabDecompiler" value={3}>Decompiler</Tab>
    <Tab bind:group={tabSet} name="tabAIDecompiler" value={4}>AI Decompiler</Tab>
    <Tab bind:group={tabSet} name="tabTools" value={5}>Tools</Tab>
    <Tab bind:group={tabSet} name="tabSettings" value={6}>Settings</Tab>
    <svelte:fragment slot="panel">
      {#if tabSet === 0}
        <ViewDashboard />
      {:else if tabSet === 1}
        <ViewInspector />
      {:else if tabSet === 2}
        <ViewDisassembler />
      {:else if tabSet === 3}
        <ViewDecompiler />
      {:else if tabSet === 4}
        <ViewAIDecompiler />
      {:else if tabSet === 5}
        <ViewTools />
      {:else if tabSet === 6}
        <ViewSettings />
      {/if}
    </svelte:fragment>
  </TabGroup>
</div>