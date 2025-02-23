<script lang="ts">
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import ViewStrings from "./ViewStrings.svelte";
  import ViewGlobals from "./ViewGlobals.svelte";
  import ViewNatives from "./ViewNatives.svelte";
  import ViewConstants from "./ViewConstants.svelte";
  import ViewInts from "./ViewInts.svelte";
  import ViewFloats from "./ViewFloats.svelte";
  import ViewHistory from "./ViewHistory.svelte";

  let tabSet: number = 0;
</script>
  

<div class="flex flex-col h-full p-2">
  <TabGroup class="h-full" regionPanel="h-[calc(100%-3rem)]" regionList="h-10">
    <Tab bind:group={tabSet} name="tabHistory" value={0}>History</Tab>
    <Tab bind:group={tabSet} name="tabStrings" value={1}>Strings</Tab>
    <Tab bind:group={tabSet} name="tabGlobals" value={2}>Globals</Tab>
    <Tab bind:group={tabSet} name="tabNatives" value={3}>Natives</Tab>
    <Tab bind:group={tabSet} name="tabConstants" value={4}>Constants</Tab>
    <Tab bind:group={tabSet} name="tabInts" value={5}>Ints</Tab>
    <Tab bind:group={tabSet} name="tabFloats" value={6}>Floats</Tab>
    <svelte:fragment slot="panel">
      {#if tabSet === 0}
        <ViewHistory />
      {:else if tabSet === 1}
        <ViewStrings />
      {:else if tabSet === 2}
        <ViewGlobals />
      {:else if tabSet === 3}
        <ViewNatives />
      {:else if tabSet === 4}
        <ViewConstants />
      {:else if tabSet === 5}
        <ViewInts />
      {:else if tabSet === 6}
        <ViewFloats />
      {/if}
    </svelte:fragment>
  </TabGroup>
</div>
