<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { typeToEdit, mainPanelTab, typesRefreshKey } from './types';
  import VirtualList from 'svelte-tiny-virtual-list';
  import ViewReferenceFinder from './Tools/ViewReferenceFinder.svelte';
  
  let typeList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];
  
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let selectedTypeIdx: number | null = null;
  let selectedTypeName: string | null = null;
  let showReferenceFinder = false;
  
  async function fetchTypeList() {
    const response = await invoke("get_type_list") as string[];
  
    if (response !== null) {
      typeList = response;
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
        typ: "class"
      }
    });

    await invoke("add_history_item", {
      item: {
        name: itemName,
        typ: "class",
        timestamp: new Date().toISOString()
      }
    });

    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: itemName,
        type: "class"
      }
    });

    window.dispatchEvent(ev);
  }

  function handleContextMenu(e: MouseEvent, idx: number) {
    e.preventDefault();
    showMenu = true;
    menuX = e.clientX;
    menuY = e.clientY;
    const itemName = filteredList[idx];
    selectedTypeName = itemName;
    selectedTypeIdx = parseInt(itemName.split('@')[1]);
  }

  async function copyTypeMenu(e) {
    e?.stopPropagation();
    if (selectedTypeName) {
      await navigator.clipboard.writeText(selectedTypeName);
    }
    showMenu = false;
  }

  function editTypeMenu(e) {
    e?.stopPropagation();
    if (selectedTypeIdx !== null) {
      typeToEdit.set(selectedTypeIdx);
      mainPanelTab.set("constructor");
    }
    showMenu = false;
  }

  function findReferencesMenu(e) {
    console.log("Selected: ", selectedTypeIdx, selectedTypeName);
    e?.stopPropagation();
    if (selectedTypeIdx !== null) {
      showReferenceFinder = true;
    }
    showMenu = false;
  }

  async function deleteTypeMenu(e) {
    e?.stopPropagation();
    showMenu = false;
  }

  function closeReferenceFinder() {
    showReferenceFinder = false;
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('click', () => showMenu = false);
    window.addEventListener('scroll', () => showMenu = false, true);
    window.addEventListener('resize', () => showMenu = false, true);
  }
  
  onMount(() => {
    fetchTypeList();
  });
  
  $: if ($typesRefreshKey !== undefined) {
    fetchTypeList();
  }
  
  $: filteredList = typeList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
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
        aria-label="Type Context Menu"
        oncontextmenu={(e) => handleContextMenu(e, index)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            const target = e.currentTarget as HTMLElement;
            const rect = target.getBoundingClientRect();
            showMenu = true;
            menuX = rect.left;
            menuY = rect.top + rect.height;
            const itemName = filteredList[index];
            selectedTypeName = itemName;
            selectedTypeIdx = parseInt(itemName.split('@')[1]);
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
  {#if showMenu && selectedTypeIdx !== null}
    <div
      class="fixed z-50 bg-surface-700 border border-surface-600 rounded shadow p-1"
      style="left: {menuX}px; top: {menuY}px; min-width: 120px"
      role="menu"
      aria-label="Type actions"
      tabindex="-1"
      onclick={() => showMenu = false}
      onkeydown={(e) => { if (e.key === "Escape") showMenu = false; }}
    >
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={copyTypeMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') copyTypeMenu(); }}>
        Copy
      </button>
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={editTypeMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') editTypeMenu(); }}>
        Edit
      </button>
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={findReferencesMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') findReferencesMenu(); }}>
        Find function references
      </button>
      <button class="block w-full text-left p-1 hover:bg-error-400/30 text-error-400 rounded"
              role="menuitem"
              tabindex="0"
              onclick={deleteTypeMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') deleteTypeMenu(); }}>
        Delete
      </button>
    </div>
  {/if}
  
  {#if showReferenceFinder && selectedTypeIdx !== null && selectedTypeName !== null}
    <ViewReferenceFinder 
      typeName={selectedTypeName}
      typeIndex={selectedTypeIdx}
      onClose={closeReferenceFinder}
    />
  {/if}
</div>