<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';
  
  let tabSet: number = 0;
  let intList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];
  
  async function fetchIntList() {
    const response = await invoke("get_int_list") as string[];
  
    if (response !== null) {
      intList = response;
    }
  }

  async function onClickButton(e: Event) {
    const target = e.target as HTMLButtonElement;
    const btnNameElement = target
    var itemName = "";

    if (btnNameElement !== null && btnNameElement.textContent !== null) {
      itemName = btnNameElement.innerText;
    }

    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: itemName,
        type: "int"
      }
    });

    window.dispatchEvent(ev);
  }

  function splitText(text: string) {
    const index = text.lastIndexOf("@");
    if (index === -1) {
      return [text, ''];
    }
    return [text.slice(0, index), text.slice(index)];
  }
  
  onMount(() => {
    fetchIntList();
  });

  $: filteredList = intList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
</script>
  
<div class="h-full">
  <div class="h-10">
    <input bind:value={searchQuery} type="text" class="input w-full variant-glass-primary text-left h-5/6" placeholder="Search" />
  </div>
  <div class="h-[calc(100%-2.5rem)] overflow-y-auto">
    <VirtualList width="100%" height="100%" itemCount={filteredList.length} itemSize={25} overscanCount={50}>
      <div slot="item" let:index let:style {style}>
        <button on:click={onClickButton} type="button" class="w-full variant-glass-primary text-left truncate">
          <span class="pointer-events-none">{splitText(filteredList[index])[0]}</span>
          <span class="text-primary-500 pointer-events-none">{splitText(filteredList[index])[1]}</span>
        </button>
      </div>
    </VirtualList>
  </div>
</div>