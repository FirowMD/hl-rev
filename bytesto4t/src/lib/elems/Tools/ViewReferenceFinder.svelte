<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import VirtualList from 'svelte-tiny-virtual-list';

  export let typeName: string = "";
  export let typeIndex: number = 0;
  export let onClose: () => void;

  let functionList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];
  let loading: boolean = true;
  let error: string = "";

  function splitText(text: string) {
    const index = text.lastIndexOf("@");
    if (index === -1) {
      return [text, ''];
    }
    return [text.slice(0, index), text.slice(index)];
  }

  async function fetchFunctionReferences() {
    try {
      loading = true;
      error = "";
      console.log("Fetching functions using type index:", typeIndex);
      const response = await invoke("find_functions_using_type_cmd", { typeIndex }) as string[];
      functionList = response || [];
    } catch (err) {
      error = (err as Error).toString();
      functionList = [];
    } finally {
      loading = false;
    }
  }

  async function onClickFunction(e: Event) {
    const target = e.target as HTMLButtonElement;
    const functionNameElement = target.querySelector("#functionName");
    const functionIndexElement = target.querySelector("#functionIndex");
    let funcName = "";
    let funcIndex = "";

    if (functionNameElement?.textContent) {
      funcName = functionNameElement.textContent;
    }

    if (functionIndexElement?.textContent) {
      funcIndex = functionIndexElement.textContent.substring(1);
    }

    const indexPart = funcIndex;
    
    if (indexPart.startsWith('native_')) {
      console.log('Native function selected, navigation not supported:', funcName + functionIndexElement?.textContent);
      return;
    }

    await invoke("set_selected_item", {
      appItem: {
        index: indexPart,
        typ: "function"
      }
    });

    const fullName = funcName + functionIndexElement?.textContent;
    await invoke("add_history_item", {
      item: {
        name: fullName,
        typ: "function",
        timestamp: new Date().toISOString()
      }
    });

    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: fullName,
        type: "function"
      }
    });

    window.dispatchEvent(ev);
  }

  fetchFunctionReferences();

  $: filteredList = functionList.filter(func => 
    func.toLowerCase().includes(searchQuery.toLowerCase())
  );
</script>

<div class="modal-backdrop fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
  <div class="modal bg-surface-800 rounded-lg shadow-xl w-11/12 max-w-2xl max-h-[80vh] flex flex-col">
    <div class="flex items-center justify-between p-4 border-b border-surface-600">
      <h2 class="text-xl font-semibold">Function References</h2>
      <button
        class="btn-icon variant-ghost-surface"
        onclick={onClose}
        aria-label="Close"
      >
        ✕
      </button>
    </div>

    <div class="px-4 py-2 bg-surface-700 border-b border-surface-600">
      <p class="text-sm text-surface-300">
        Functions using type: <span class="text-primary-400 font-mono">{typeName}</span>
      </p>
    </div>

    <div class="p-4 border-b border-surface-600">
      <input
        bind:value={searchQuery}
        type="text"
        class="input w-full"
        placeholder="Search functions..."
        disabled={loading}
      />
    </div>

    <div class="flex-1 overflow-hidden">
      {#if loading}
        <div class="flex items-center justify-center h-32">
          <p class="text-surface-400">Loading function references...</p>
        </div>
      {:else if error}
        <div class="p-4 text-error-400">
          <p>Error: {error}</p>
        </div>
      {:else if filteredList.length === 0}
        <div class="p-4 text-surface-400 text-center">
          {#if functionList.length === 0}
            <p>No functions reference this type.</p>
          {:else}
            <p>No functions match your search.</p>
          {/if}
        </div>
      {:else}
        <div class="h-full p-4">
          <VirtualList 
            width="100%" 
            height="100%" 
            itemCount={filteredList.length} 
            itemSize={35} 
            overscanCount={10}
          >
            <div slot="item" let:index let:style {style} class="pr-2">
              {#if splitText(filteredList[index])[1].startsWith('@native_')}
                <div class="w-full p-2 bg-surface-600 rounded text-xs truncate text-left opacity-75">
                  <div class="flex flex-row justify-start w-full">
                    <span class="text-warning-400">[Native]</span>
                    <span class="ml-2">{splitText(filteredList[index])[0]}</span>
                    <span class="text-tertiary-50 ml-auto">{splitText(filteredList[index])[1]}</span>
                  </div>
                </div>
              {:else}
                <button 
                  onclick={onClickFunction} 
                  type="button" 
                  class="btn w-full preset-filled-surface-500 text-xs truncate text-left justify-start"
                >
                  <div class="flex flex-row justify-start w-full">
                    <span id="functionName" class="pointer-events-none">{splitText(filteredList[index])[0]}</span>
                    <span id="functionIndex" class="text-tertiary-50 pointer-events-none ml-auto">{splitText(filteredList[index])[1]}</span>
                  </div>
                </button>
              {/if}
            </div>
          </VirtualList>
        </div>
      {/if}
    </div>

    <div class="p-4 border-t border-surface-600 bg-surface-700">
      <p class="text-xs text-surface-400">
        Found {functionList.length} function{functionList.length !== 1 ? 's' : ''} 
        {#if searchQuery}(showing {filteredList.length} filtered){/if}
      </p>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    backdrop-filter: blur(2px);
  }
</style>