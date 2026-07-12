<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { nativeToEdit, mainPanelTab } from './types';
  import VirtualList from 'svelte-tiny-virtual-list';
  
  let tabSet: number = 0;
  let nativeList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];
  
  // Context menu state
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let selectedNativeIdx: number | null = null;
  let selectedNativeName: string | null = null;
  
  async function fetchNativeList() {
    const response = await invoke("get_native_list") as string[];
  
    if (response !== null) {
      nativeList = response;
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
    const nativeIndex = itemName.substring(lastAtIndex + 1).split(' ')[0];

    await invoke("set_selected_item", {
      appItem: {
        index: nativeIndex,
        typ: "native"
      }
    });

    // Add history item
    await invoke("add_history_item", {
      item: {
        name: itemName,
        typ: "native",
        timestamp: new Date().toISOString()
      }
    });

    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: itemName,
        type: "native"
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
    selectedNativeName = name + idxStr;
    selectedNativeIdx = parseInt(idxStr.replace("@", "").split(' ')[0]);
  }

  async function copyNativeMenu(e?: Event) {
    e?.stopPropagation();
    if (selectedNativeName) {
      await navigator.clipboard.writeText(selectedNativeName);
    }
    showMenu = false;
  }

  function editNativeMenu(e?: Event) {
    e?.stopPropagation();
    if (selectedNativeIdx !== null) {
      nativeToEdit.set(selectedNativeIdx);
      mainPanelTab.set("constructor");
    }
    showMenu = false;
  }

  async function deleteNativeMenu(e?: Event) {
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
    fetchNativeList();
  });
  
  $: filteredList = nativeList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
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
    <VirtualList width="100%" height="100%" itemCount={filteredList.length} itemSize={27} overscanCount={50}>
      <div 
        slot="item" let:index let:style {style}
        class="relative"
        role="menuitem"
        tabindex="0"
        aria-label="Native Context Menu"
        oncontextmenu={(e) => handleContextMenu(e, index)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            const target = e.currentTarget as HTMLElement;
            const rect = target.getBoundingClientRect();
            showMenu = true;
            menuX = rect.left;
            menuY = rect.top + rect.height;
            const [name, idxStr] = splitText(filteredList[index]);
            selectedNativeName = name + idxStr;
            selectedNativeIdx = parseInt(idxStr.replace("@", "").split(' ')[0]);
            e.preventDefault();
          }
        }}
      >
        <button onclick={onClickButton} type="button" class="explorer-row truncate">
          <div class="flex flex-row justify-start w-full">
            <span id="functionName" class="pointer-events-none">{splitText(filteredList[index])[0]}</span>
            <span id="functionIndex" class="explorer-index pointer-events-none">{splitText(filteredList[index])[1]}</span>
          </div>
        </button>
      </div>
    </VirtualList>
  </div>
  {#if showMenu && selectedNativeIdx !== null}
    <div
      class="fixed z-50 bg-surface-700 border border-surface-600 rounded shadow p-1"
      style="left: {menuX}px; top: {menuY}px; min-width: 120px"
      role="menu"
      aria-label="Native actions"
      tabindex="-1"
      onclick={() => showMenu = false}
      onkeydown={(e) => { if (e.key === "Escape") showMenu = false; }}
    >
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={copyNativeMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') copyNativeMenu(); }}>
        Copy
      </button>
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={editNativeMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') editNativeMenu(); }}>
        Edit
      </button>
      <button class="block w-full text-left p-1 hover:bg-error-400/30 text-error-400 rounded"
              role="menuitem"
              tabindex="0"
              onclick={deleteNativeMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') deleteNativeMenu(); }}>
        Delete
      </button>
    </div>
  {/if}
</div>
