<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';
  import type { HistoryItem } from './types';

  let historyItems = $state<HistoryItem[]>([]);
  let searchQuery = $state("");

  // Context menu state
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let selectedHistoryItem: HistoryItem | null = null;

  async function loadHistory() {
    try {
      historyItems = await invoke("get_history_items") as HistoryItem[];
    } catch (error) {
      console.error("Failed to load history:", error);
    }
  }

  async function bytecodeItemSelectedHandler(e: Event) {
    const ev = e as CustomEvent<{name: string, type: string}>;
    
    const newItem = {
      name: ev.detail.name,
      typ: ev.detail.type,
      timestamp: new Date().toISOString()
    };

    try {
      await invoke("add_history_item", { item: newItem });
      await loadHistory();
    } catch (error) {
      console.error("Failed to add history item:", error);
    }
  }

  function formatTimestamp(isoString: string): string {
    return new Date(isoString).toLocaleTimeString();
  }

  async function onClickButton(e: Event) {
    const target = e.target as HTMLButtonElement;
    const itemElement = target.closest('button');
    
    if (!itemElement?.dataset.name || !itemElement?.dataset.type) return;

    const itemName = itemElement.dataset.name;
    const itemType = itemElement.dataset.type;

    const index = itemName.split('@').pop();
    if (!index) return;

    await invoke("set_selected_item", {
      appItem: {
        index,
        typ: itemType
      }
    });

    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: itemName,
        type: itemType
      }
    });

    window.dispatchEvent(ev);
  }

  function handleContextMenu(e: MouseEvent, idx: number) {
    e.preventDefault();
    showMenu = true;
    menuX = e.clientX;
    menuY = e.clientY;
    selectedHistoryItem = filteredItems[idx];
  }

  async function copyHistoryMenu(e) {
    e?.stopPropagation();
    if (selectedHistoryItem) {
      await navigator.clipboard.writeText(selectedHistoryItem.name);
    }
    showMenu = false;
  }

  function editHistoryMenu(e) {
    e?.stopPropagation();
    // TODO: Implement edit logic
    showMenu = false;
  }

  async function deleteHistoryMenu(e) {
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
    loadHistory();
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    return () => {
      window.removeEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    };
  });

  const filteredItems = $derived(historyItems.filter(item => 
    item.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
    item.typ.toLowerCase().includes(searchQuery.toLowerCase())
  ));
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
    <VirtualList 
      width="100%" 
      height="100%" 
      itemCount={filteredItems.length} 
      itemSize={27} 
      overscanCount={50}
    >
      <div 
        slot="item" let:index let:style {style}
        class="relative"
        role="menuitem"
        tabindex="0"
        aria-label="History Context Menu"
        oncontextmenu={(e) => handleContextMenu(e, index)}
        onkeydown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            const target = e.currentTarget as HTMLElement;
            const rect = target.getBoundingClientRect();
            showMenu = true;
            menuX = rect.left;
            menuY = rect.top + rect.height;
            selectedHistoryItem = filteredItems[index];
            e.preventDefault();
          }
        }}
      >
        <button 
          onclick={onClickButton}
          type="button" 
          class="btn w-full preset-filled-surface-500 text-xs truncate"
          data-name={filteredItems[index].name}
          data-type={filteredItems[index].typ}
        >
          <div class="flex flex-row justify-start gap-2 w-full">
            <span class="text-primary-50 pointer-events-none">[{filteredItems[index].typ}]</span>
            <span class="pointer-events-none">{filteredItems[index].name}</span>
            <span class="ml-auto text-secondary-50 pointer-events-none">
              {formatTimestamp(filteredItems[index].timestamp)}
            </span>
          </div>
        </button>
      </div>
    </VirtualList>
  </div>
  {#if showMenu && selectedHistoryItem !== null}
    <div
      class="fixed z-50 bg-surface-700 border border-surface-600 rounded shadow p-1"
      style="left: {menuX}px; top: {menuY}px; min-width: 120px"
      role="menu"
      aria-label="History actions"
      tabindex="-1"
      onclick={() => showMenu = false}
      onkeydown={(e) => { if (e.key === "Escape") showMenu = false; }}
    >
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={copyHistoryMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') copyHistoryMenu(); }}>
        Copy
      </button>
      <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
              role="menuitem"
              tabindex="0"
              onclick={editHistoryMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') editHistoryMenu(); }}>
        Edit
      </button>
      <button class="block w-full text-left p-1 hover:bg-error-400/30 text-error-400 rounded"
              role="menuitem"
              tabindex="0"
              onclick={deleteHistoryMenu}
              onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') deleteHistoryMenu(); }}>
        Delete
      </button>
    </div>
  {/if}
</div>