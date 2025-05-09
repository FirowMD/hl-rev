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
  import type { FileData, BytecodeItemSelectedEvent } from '../elems/types';

  let fileData = $state<FileData | undefined>();
  let activeTab = $state('dashboard');

  const isLargeFile = $derived((fileData?.size ?? 0) > 1024 * 1024);
  
  let hexScrollPosition = $state(0);
  let hexTargetOffset = $state(-1);
  let lastKnownFoffset = $state<number | null>(null);

  const tabs = [
    { id: 'dashboard', label: 'Dashboard', component: ViewDashboard },
    { id: 'inspector', label: 'Inspector', component: ViewInspector },
    { id: 'disassembler', label: 'Disassembler', component: ViewDisassembler },
    { id: 'decompiler', label: 'Decompiler', component: ViewDecompiler },
    { id: 'aidecompiler', label: 'AI Decompiler', component: ViewAIDecompiler },
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
</script>

<div class="w-full h-full bg-surface-950">
  <div class="flex border-b border-surface-700 truncate overflow-x-auto">
    {#each tabs as tab}
      <button
        class="px-4 py-1 {activeTab === tab.id ? 'bg-surface-800 border-b-2 border-primary-500' : 'hover:bg-surface-800/50'}"
        onclick={() => activeTab = tab.id}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="px-2 py-1 h-[calc(100%-3rem)] overflow-hidden">
    {#each tabs as tab}
      {#if activeTab === tab.id}
        <svelte:component this={tab.component} />
      {/if}
    {/each}
  </div>
</div>