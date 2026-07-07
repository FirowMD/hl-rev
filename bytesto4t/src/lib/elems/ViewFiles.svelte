<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';

  let tabSet: number = 0;
  let fileList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];

  // Context menu state
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let selectedFileIdx: number | null = null;
  let selectedFileName: string | null = null;

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

    await invoke("add_history_item", {
      item: {
        name: itemName,
        typ: "file",
        timestamp: new Date().toISOString()
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

  function handleContextMenu(e: MouseEvent, idx: number) {
    e.preventDefault();
    showMenu = true;
    menuX = e.clientX;
    menuY = e.clientY;
    // Parse name and index out for action
    const itemName = filteredList[idx];
    selectedFileName = itemName;
    const lastAtIndex = itemName.lastIndexOf('@');
    selectedFileIdx = parseInt(itemName.substring(lastAtIndex + 1));
  }

  async function copyFileMenu(e) {
    e?.stopPropagation();
    if (selectedFileName) {
      await navigator.clipboard.writeText(selectedFileName);
    }
    showMenu = false;
  }

  async function deleteFileMenu(e) {
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
    fetchFileList();
  });

  $: filteredList = fileList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
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
        aria-label="File Context Menu"
        oncontextmenu={(e) => handleContextMenu(e, index)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            const target = e.currentTarget as HTMLElement;
            const rect = target.getBoundingClientRect();
            showMenu = true;
            menuX = rect.left;
            menuY = rect.top + rect.height;
            const itemName = filteredList[index];
            selectedFileName = itemName;
            const lastAtIndex = itemName.lastIndexOf('@');
            selectedFileIdx = parseInt(itemName.substring(lastAtIndex + 1));
            e.preventDefault();
          }
        }}
      >
        <button onclick={onClickButton} type="button" class="btn w-full preset-filled-surface-500 text-xs truncate">
          <div class="flex flex-row justify-start w-full">
            <span class="pointer-events-none">{filteredList[index].split('@')[0]}</span>
            <span class="text-tertiary-50 pointer-events-none">@{filteredList[index].split('@')[1]}</span>
          </div>
        </button>
      </div>
    </VirtualList>
  </div>
  {#if showMenu && selectedFileIdx !== null}
    <div
      class="fixed z-50 bg-surface-700 border border-surface-600 rounded shadow p-1"
      style="left: {menuX}px; top: {menuY}px; min-width: 120px"
      role="menu"
      aria-label="File actions"
      tabindex="-1"
      onclick={() => showMenu = false}
      onkeydown={(e) => { if (e.key === "Escape") showMenu = false; }}
    >
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={copyFileMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') copyFileMenu(); }}>
        Copy
      </button>
      <button class="block w-full text-left p-1 hover:bg-error-400/30 text-error-400 rounded"
              role="menuitem"
              tabindex="0"
              onclick={deleteFileMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') deleteFileMenu(); }}>
        Delete
      </button>
    </div>
  {/if}
</div>