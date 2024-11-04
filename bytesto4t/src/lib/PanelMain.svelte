<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import ViewDashboard from "./ViewDashboard.svelte";
  import ViewInspector from "./ViewInspector.svelte";
  import ViewDisassembler from "./ViewDisassembler.svelte";
  import ViewDecompiler from "./ViewDecompiler.svelte";
  import ViewHex from "./ViewHex.svelte";
  import ViewTools from "./ViewTools.svelte";
  import ViewSettings from "./ViewSettings.svelte";

  let fileData = $state<{
    buffer: Uint8Array,
    size: number,
    name: string
  } | undefined>();

  const isLargeFile = $derived((fileData?.size ?? 0) > 1024 * 1024);
  
  let hexScrollPosition = $state(0);

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

  onMount(async () => {
    await loadFile();
  });

  let tabSet = $state(0);
</script>

<div class="flex flex-col h-full p-2">
  <TabGroup class="h-full" regionPanel="h-[calc(100%-3rem)]" regionList="h-10">
    <Tab bind:group={tabSet} name="tabDashboard" value={0}>Dashboard</Tab>
    <Tab bind:group={tabSet} name="tabInspector" value={1}>Inspector</Tab>
    <Tab bind:group={tabSet} name="tabDisassembler" value={2}>Disassembler</Tab>
    <Tab bind:group={tabSet} name="tabDecompiler" value={3}>Decompiler</Tab>
    <Tab bind:group={tabSet} name="tabHex" value={4}>
      Hex {#if fileData}({(fileData.size / 1024).toFixed(1)} KB){/if}
    </Tab>
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
      {:else if tabSet === 4 && fileData}
        <ViewHex 
          data={fileData.buffer} 
          bytesPerRow={16}
          scrollPosition={hexScrollPosition}
          on:scroll={handleHexScroll}
        />
      {:else if tabSet === 5}
        <ViewTools />
      {:else if tabSet === 6}
        <ViewSettings />
      {/if}
    </svelte:fragment>
  </TabGroup>
</div>