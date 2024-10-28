<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';
  
  let typeList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];
  
  async function fetchTypeList() {
    const response = await invoke("get_type_list") as string[];
  
    if (response !== null) {
      typeList = response;
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
        type: "class"
      }
    });

    window.dispatchEvent(ev);
  }
  
  onMount(() => {
    fetchTypeList();
  });
  
  $: filteredList = typeList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
</script>

<div class="h-full">
  <div class="h-10">
    <input bind:value={searchQuery} type="text" class="input w-full variant-glass-primary text-left h-5/6" placeholder="Search" />
  </div>
  <div class="h-[calc(100%-2.5rem)] overflow-y-auto">
    <VirtualList width="100%" height="100%" itemCount={filteredList.length} itemSize={25} overscanCount={50}>
      <div slot="item" let:index let:style {style}>
        <button on:click={onClickButton} type="button" class="w-full variant-glass-primary text-left truncate">{filteredList[index]}</button>
      </div>
    </VirtualList>
  </div>
</div>