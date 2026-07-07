<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { byteToEdit, mainPanelTab, bytesRefreshKey } from './types';
  import VirtualList from 'svelte-tiny-virtual-list';
  
  let tabSet: number = 0;
  let bytesList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];
  
  // Context menu state
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let selectedByteIdx: number | null = null;
  let selectedByteName: string | null = null;
  
  async function fetchBytesList() {
    const response = await invoke("get_bytes_list") as string[];
  
    if (response !== null) {
      bytesList = response;
    }
  }

  async function onClickButton(e: Event) {
    const target = e.target as HTMLButtonElement;
    const btnNameElement = target;
    var itemName = "";

    if (btnNameElement !== null && btnNameElement.textContent !== null) {
      itemName = btnNameElement.innerText;
    }

    await invoke("set_selected_item", {
      appItem: {
        index: itemName.split('@')[1],
        typ: "bytes"
      }
    });

    await invoke("add_history_item", {
      item: {
        name: itemName,
        typ: "bytes",
        timestamp: new Date().toISOString()
      }
    });

    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: itemName,
        type: "bytes"
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

  function handleContextMenu(e: MouseEvent, idx: number) {
    e.preventDefault();
    showMenu = true;
    menuX = e.clientX;
    menuY = e.clientY;
    // Parse name and index out for action
    const [name, idxStr] = splitText(filteredList[idx]);
    selectedByteName = name + idxStr;
    selectedByteIdx = parseInt(idxStr.replace("@", ""));
  }

  async function copyByteMenu(e) {
    e?.stopPropagation();
    if (selectedByteName) {
      await navigator.clipboard.writeText(selectedByteName);
    }
    showMenu = false;
  }

  function editByteMenu(e) {
    e?.stopPropagation();
    if (selectedByteIdx !== null) {
      byteToEdit.set(selectedByteIdx);
      mainPanelTab.set("constructor");
    }
    showMenu = false;
  }

  async function deleteByteMenu(e) {
    e?.stopPropagation();
    // TODO: Implement delete logic
    showMenu = false;
  }

  // Dismiss menu on global click/scroll/resize
  if (typeof window !== 'undefined') {
    window.addEventListener('click', () => showMenu = false);
    window.addEventListener('scroll', () => showMenu = false, true);
    window.addEventListener('resize', () => showMenu = false, true);
  }

  onMount(() => {
    fetchBytesList();
  });
  
  // Refresh bytes list on global signal
  $: if ($bytesRefreshKey !== undefined) {
    fetchBytesList();
  }
  
  $: filteredList = bytesList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
</script>

<div class="h-full">
  <div class="h-10">
    <input
      bind:value={searchQuery}
      type="text"
      class="input w-full text-left h-5/6 focus:outline-none"
      placeholder="Search"
    />
  </div>
  <div class="h-[calc(100%-2.5rem)] overflow-y-auto">
    <VirtualList width="100%" height="100%" itemCount={filteredList.length} itemSize={27} overscanCount={50}>
      <div 
        slot="item" let:index let:style {style}
        class="relative"
        role="menuitem"
        tabindex="0"
        aria-label="Bytes Context Menu"
        oncontextmenu={(e) => handleContextMenu(e, index)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            const target = e.currentTarget as HTMLElement;
            const rect = target.getBoundingClientRect();
            showMenu = true;
            menuX = rect.left;
            menuY = rect.top + rect.height;
            const [name, idxStr] = splitText(filteredList[index]);
            selectedByteName = name + idxStr;
            selectedByteIdx = parseInt(idxStr.replace("@", ""));
            e.preventDefault();
          }
        }}
      >
        <button onclick={onClickButton} type="button" class="btn w-full preset-filled-surface-500 text-xs truncate">
          <div class="flex flex-row justify-start w-full">
            <span id="functionName" class="pointer-events-none font-mono">{splitText(filteredList[index])[0]}</span>
            <span id="functionIndex" class="text-tertiary-50 pointer-events-none">{splitText(filteredList[index])[1]}</span>
          </div>
        </button>
      </div>
    </VirtualList>
  </div>
  {#if showMenu && selectedByteIdx !== null}
    <div
      class="fixed z-50 bg-surface-700 border border-surface-600 rounded shadow p-1"
      style="left: {menuX}px; top: {menuY}px; min-width: 120px"
      role="menu"
      aria-label="Bytes actions"
      tabindex="-1"
      onclick={() => showMenu = false}
      onkeydown={(e) => { if (e.key === "Escape") showMenu = false; }}
    >
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={copyByteMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') copyByteMenu(); }}>
        Copy
      </button>
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={editByteMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') editByteMenu(); }}>
        Edit
      </button>
      <button class="block w-full text-left p-1 hover:bg-error-400/30 text-error-400 rounded"
              role="menuitem"
              tabindex="0"
              onclick={deleteByteMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') deleteByteMenu(); }}>
        Delete
      </button>
    </div>
  {/if}
</div>