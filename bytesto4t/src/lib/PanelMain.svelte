<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { Pane, Splitpanes } from "svelte-splitpanes";
  import { Accordion, AccordionItem } from "@skeletonlabs/skeleton";
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import ViewDashboard from "./ViewDashboard.svelte";
  import ViewInspector from "./ViewInspector.svelte";
  import ViewDisassembler from "./ViewDisassembler.svelte";
  import ViewDecompiler from "./ViewDecompiler.svelte";
  import ViewTools from "./ViewTools.svelte";
  import ViewSettings from "./ViewSettings.svelte";

  let tabSet: number = 0;
</script>

<div class="flex flex-col h-full p-2">
  <TabGroup class="h-full" regionPanel="h-[calc(100%-3rem)]" regionList="h-10">
    <Tab bind:group={tabSet} name="tabDashboard" value={0}>Dashboard</Tab>
    <Tab bind:group={tabSet} name="tabInspector" value={1}>Inspector</Tab>
    <Tab bind:group={tabSet} name="tabDisassembler" value={2}>Disassembler</Tab>
    <Tab bind:group={tabSet} name="tabDecompiler" value={3}>Decompiler</Tab>
    <Tab bind:group={tabSet} name="tabHex" value={4}>Hex</Tab>
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
        <pre>
          Not implemented yet
        </pre>
      {:else if tabSet === 5}
        <ViewTools />
      {:else if tabSet === 6}
        <ViewSettings />
      {/if}
    </svelte:fragment>
  </TabGroup>
</div>