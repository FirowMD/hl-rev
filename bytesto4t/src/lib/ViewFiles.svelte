<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';

  let tabSet: number = 0;
  let fileList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];

  async function fetchFileList() {
    try {
      const response = await invoke("get_file_list") as string[];
      fileList = response || [];  // Use empty array if response is null/undefined
    } catch (error) {
      console.error('Failed to fetch file list:', error);
      fileList = [];  // Set empty array on error
    }
  }

  async function onClickButton(e: Event) {
    const target = e.target as HTMLButtonElement;
    const btnNameElement = target;
    var itemName = "";

    if (btnNameElement !== null && btnNameElement.textContent !== null) {
      itemName = btnNameElement.innerText;
    }

    const lastAtIndex = itemName.lastIndexOf('@');
    const fileIndex = itemName.substring(lastAtIndex + 1);

    await invoke("set_selected_item", {
      appItem: {
        index: fileIndex,
        typ: "file"
      }
    });

    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: itemName,
        type: "file"
      }
    });

    window.dispatchEvent(ev);
  }

  onMount(() => {
    fetchFileList();
  });

  $: filteredList = fileList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
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