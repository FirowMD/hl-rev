<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import VirtualList from 'svelte-tiny-virtual-list';

  interface HistoryItem {
    name: string;
    typ: string;
    timestamp: string;
  }

  let historyItems = $state<HistoryItem[]>([]);
  let searchQuery = $state("");

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
      class="input w-full variant-glass-primary text-left h-5/6" 
      placeholder="Search" 
    />
  </div>
  <div class="h-[calc(100%-2.5rem)] overflow-y-auto">
    <VirtualList 
      width="100%" 
      height="100%" 
      itemCount={filteredItems.length} 
      itemSize={25} 
      overscanCount={50}
    >
      <div slot="item" let:index let:style {style}>
        <button 
          onclick={onClickButton}
          type="button" 
          class="w-full variant-glass-primary text-left truncate flex justify-between items-center"
          data-name={filteredItems[index].name}
          data-type={filteredItems[index].typ}
        >
          <div class="flex-1 truncate">
            <span class="text-primary-400">[{filteredItems[index].typ}]</span>
            <span class="ml-2">{filteredItems[index].name}</span>
          </div>
          <span class="text-sm text-secondary-400">
            {formatTimestamp(filteredItems[index].timestamp)}
          </span>
        </button>
      </div>
    </VirtualList>
  </div>
</div> 