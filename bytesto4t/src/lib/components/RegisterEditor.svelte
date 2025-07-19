<script lang="ts">
  import type { RegisterPopover, TypeInfo } from '../types/function-types';
  import VirtualList from 'svelte-tiny-virtual-list';
  import { createEventDispatcher } from 'svelte';

  export let registers: number[];
  export let availableTypes: TypeInfo[];
  export let registerPopovers: RegisterPopover[];

  const dispatch = createEventDispatcher();

  function addRegister() {
    dispatch('add');
  }

  function removeRegister(idx: number) {
    dispatch('remove', { index: idx });
  }

  function setRegister(idx: number, typeIdx: number) {
    dispatch('set', { index: idx, value: typeIdx });
  }
</script>

<div>
  <div class="mb-2 flex items-center gap-2">
    <span class="font-mono text-xs">Registers</span>
    <button class="btn btn-xs" type="button" on:click={addRegister}>+</button>
  </div>

  <div class="flex flex-col gap-2">
    {#each registers as reg, idx (idx)}
      <div class="flex items-center gap-2">
        <div class="relative" style="z-index:5;">
          <button
            type="button"
            class="input w-52 text-left"
            on:click={() => {registerPopovers[idx].open = true; registerPopovers[idx].query = "";}}
          >
            {#if availableTypes.length}
              {availableTypes.find(t => t.idx === reg)?.name ?? reg}
            {:else}
              <span class="opacity-50">Pick type...</span>
            {/if}
          </button>

          {#if registerPopovers[idx].open}
            <div class="popover" bind:this={registerPopovers[idx].ref} style="width: 210px;">
              <input
                class="input w-full mb-2"
                type="text"
                placeholder="Search types..."
                bind:value={registerPopovers[idx].query}
              />
              <div style="height: 210px;">
                {#if registerPopovers[idx].filteredTypes.length > 0}
                  <VirtualList
                    width="100%"
                    height="100%"
                    itemCount={registerPopovers[idx].filteredTypes.length}
                    itemSize={30}
                    overscanCount={5}
                  >
                    <div slot="item" let:index let:style={style}>
                      <button
                        class="w-full text-left px-2 py-1 hover:bg-primary-400/30 block"
                        on:click={() => {
                          setRegister(idx, registerPopovers[idx].filteredTypes[index].idx);
                          registerPopovers[idx].open = false;
                        }}
                        style={style}
                      >
                        <span class="font-mono">{registerPopovers[idx].filteredTypes[index].idx}</span>
                        <span class="ml-2">{registerPopovers[idx].filteredTypes[index].name}</span>
                      </button>
                    </div>
                  </VirtualList>
                {:else}
                  <div class="px-2 py-1 text-xs text-secondary-400">No results.</div>
                {/if}
              </div>
            </div>
          {/if}
        </div>
        <button
          class="btn btn-xs"
          type="button"
          on:click={() => removeRegister(idx)}
          disabled={registers.length <= 1}
        >–</button>
      </div>
    {/each}
  </div>
</div>

<style>
.input, .btn, .popover {
  /* ...existing code... */
}
</style>
