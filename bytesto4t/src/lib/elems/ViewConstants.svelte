<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { constantToEdit, mainPanelTab } from './types';
  import VirtualList from 'svelte-tiny-virtual-list';
  
  let tabSet: number = 0;
  let constantList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];
  
  // Context menu state
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let selectedConstantIdx: number | null = null;
  let selectedConstantName: string | null = null;
  
  async function fetchConstantList() {
    const response = await invoke("get_constant_list") as string[];
  
    if (response !== null) {
      constantList = response;
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
        typ: "constant"
      }
    });

    // Add history item
    await invoke("add_history_item", {
      item: {
        name: itemName,
        typ: "constant",
        timestamp: new Date().toISOString()
      }
    });

    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: itemName,
        type: "constant"
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
    selectedConstantName = name + idxStr;
    selectedConstantIdx = parseInt(idxStr.replace("@", ""));
  }

  async function copyConstantMenu(e?: Event) {
    e?.stopPropagation();
    if (selectedConstantName) {
      await navigator.clipboard.writeText(selectedConstantName);
    }
    showMenu = false;
  }

  function editConstantMenu(e?: Event) {
    e?.stopPropagation();
    if (selectedConstantIdx !== null) {
      constantToEdit.set(selectedConstantIdx);
      mainPanelTab.set("constructor");
    }
    showMenu = false;
  }

  async function deleteConstantMenu(e?: Event) {
    e?.stopPropagation();
    // TODO: Implement delete logic
    showMenu = false;
  }

  const closeMenu = () => showMenu = false;

  onMount(() => {
    window.addEventListener('click', closeMenu);
    window.addEventListener('scroll', closeMenu, true);
    window.addEventListener('resize', closeMenu, true);

    return () => {
      window.removeEventListener('click', closeMenu);
      window.removeEventListener('scroll', closeMenu, true);
      window.removeEventListener('resize', closeMenu, true);
    };
  });  
  onMount(() => {
    fetchConstantList();
  });
  
  $: filteredList = constantList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
</script>
  
<div class="h-full">
  <div class="h-8">
    <input
      bind:value={searchQuery}
      type="text"
      class="input explorer-search w-full text-left focus:outline-none"
      placeholder="Search"
    />
  </div>
  <div class="h-[calc(100%-2rem)] overflow-y-auto">
    <VirtualList width="100%" height="100%" itemCount={filteredList.length} itemSize={25} overscanCount={50}>
      <div 
        slot="item" let:index let:style {style}
        class="relative"
        role="menuitem"
        tabindex="0"
        aria-label="Constant Context Menu"
        oncontextmenu={(e) => handleContextMenu(e, index)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            const target = e.currentTarget as HTMLElement;
            const rect = target.getBoundingClientRect();
            showMenu = true;
            menuX = rect.left;
            menuY = rect.top + rect.height;
            const [name, idxStr] = splitText(filteredList[index]);
            selectedConstantName = name + idxStr;
            selectedConstantIdx = parseInt(idxStr.replace("@", ""));
            e.preventDefault();
          }
        }}
      >
        <button onclick={onClickButton} type="button" class="explorer-row truncate">
          <span class="pointer-events-none">{splitText(filteredList[index])[0]}</span>
          <span class="explorer-index pointer-events-none">{splitText(filteredList[index])[1]}</span>
        </button>
      </div>
    </VirtualList>
  </div>
  {#if showMenu && selectedConstantIdx !== null}
    <div
      class="fixed z-50 bg-surface-700 border border-surface-600 rounded shadow p-1"
      style="left: {menuX}px; top: {menuY}px; min-width: 120px"
      role="menu"
      aria-label="Constant actions"
      tabindex="-1"
      onclick={() => showMenu = false}
      onkeydown={(e) => { if (e.key === "Escape") showMenu = false; }}
    >
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={copyConstantMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') copyConstantMenu(); }}>
        Copy
      </button>
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={editConstantMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') editConstantMenu(); }}>
        Edit
      </button>
      <button class="block w-full text-left p-1 hover:bg-error-400/30 text-error-400 rounded"
              role="menuitem"
              tabindex="0"
              onclick={deleteConstantMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') deleteConstantMenu(); }}>
        Delete
      </button>
    </div>
  {/if}
</div>
