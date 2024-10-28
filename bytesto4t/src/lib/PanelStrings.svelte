<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import ViewStrings from "./ViewStrings.svelte";
  import ViewGlobals from "./ViewGlobals.svelte";
  import ViewNatives from "./ViewNatives.svelte";
  import ViewConstants from "./ViewConstants.svelte";
  import ViewInts from "./ViewInts.svelte";
  import ViewFloats from "./ViewFloats.svelte";

  let tabSet: number = 0;
</script>
  

<div class="flex flex-col h-full p-2">
  <TabGroup class="h-full" regionPanel="h-[calc(100%-3rem)]" regionList="h-10">
    <Tab bind:group={tabSet} name="tabStrings" value={0}>Strings</Tab>
    <Tab bind:group={tabSet} name="tabGlobals" value={1}>Globals</Tab>
    <Tab bind:group={tabSet} name="tabNatives" value={2}>Natives</Tab>
    <Tab bind:group={tabSet} name="tabConstants" value={3}>Constants</Tab>
    <Tab bind:group={tabSet} name="tabInts" value={4}>Ints</Tab>
    <Tab bind:group={tabSet} name="tabFloats" value={5}>Floats</Tab>
    <svelte:fragment slot="panel">
      {#if tabSet === 0}
        <ViewStrings />
      {:else if tabSet === 1}
        <ViewGlobals />
      {:else if tabSet === 2}
        <ViewNatives />
      {:else if tabSet === 3}
        <ViewConstants />
      {:else if tabSet === 4}
        <ViewInts />
      {:else if tabSet === 5}
        <ViewFloats />
      {/if}
    </svelte:fragment>
  </TabGroup>
</div>
