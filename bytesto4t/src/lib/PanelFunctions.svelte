<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import ViewFunctions from "./ViewFunctions.svelte";
  import ViewTypes from "./ViewTypes.svelte";
  import ViewFiles from "./ViewFiles.svelte";

  let tabSet: number = 0;
</script>


<div class="flex flex-col h-full p-2">
  <TabGroup class="h-full" regionPanel="h-[calc(100%-3rem)]" regionList="h-10">
    <Tab bind:group={tabSet} name="tabFunctions" value={0}>Functions</Tab>
    <Tab bind:group={tabSet} name="tabTypes" value={1}>Classes</Tab>
    <Tab bind:group={tabSet} name="tabFiles" value={2}>Files</Tab>
    <svelte:fragment slot="panel">
      {#if tabSet === 0}
        <ViewFunctions />
      {:else if tabSet === 1}
        <ViewTypes />
      {:else if tabSet === 2}
        <ViewFiles />
      {/if}
    </svelte:fragment>
  </TabGroup>
</div>
