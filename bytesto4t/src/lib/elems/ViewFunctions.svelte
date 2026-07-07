<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, createEventDispatcher } from "svelte";
  export let active: boolean = false;
  let alreadyLoaded = false;
  import VirtualList from 'svelte-tiny-virtual-list';
  import { functionToEdit, mainPanelTab } from './types';
  // Removed local import of ViewFunctionConstructor:
  // import ViewFunctionConstructor from './ViewFunctionConstructor.svelte';
  
  import { functionsRefreshKey } from './types';
  
  let functionList: string[] = [];
  let searchQuery: string = "";
  let filteredList: string[] = [];
  
  let showMenu = false;
  let menuX = 0;
  let menuY = 0;
  let selectedFunctionIdx: number | null = null;
  let selectedFunctionName: string | null = null;

  const dispatch = createEventDispatcher();
  
  async function fetchFunctionList() {
    const response = await invoke("get_function_list") as string[];

    if (response !== null) {
      functionList = response;
    }

    functionList.sort((a, b) => {
      const aNumber = parseInt(a.split("@")[1]);
      const bNumber = parseInt(b.split("@")[1]);
      return aNumber - bNumber;
    });
  }

  async function onClickButton(e: Event) {
    const target = e.target as HTMLButtonElement;
    const functionNameElement = target.querySelector("#functionName");
    const functionIndexElement = target.querySelector("#functionIndex");
    var funcName = "";
    var funcIndex = "";

    if (functionNameElement !== null && functionNameElement.textContent !== null) {
      funcName = functionNameElement.textContent;
    }

    if (functionIndexElement !== null && functionIndexElement.textContent !== null) {
      funcIndex = functionIndexElement.textContent.substring(1);
    }

    console.log("findex: `" + funcIndex + "`");
    await invoke("set_selected_item", {
      appItem: {
        index: funcIndex,
        typ: "function"
      }
    });

    console.log("fullName: `" + funcName + functionIndexElement?.textContent + "`");
    await invoke("add_history_item", {
      item: {
        name: funcName + functionIndexElement?.textContent,
        typ: "function",
        timestamp: new Date().toISOString()
      }
    });

    console.log("fullName: `" + funcName + functionIndexElement?.textContent + "`");
    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: funcName + functionIndexElement?.textContent,
        type: "function"
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

  onMount(() => {
    fetchFunctionList();
  });

  $: filteredList = functionList.filter(func => func.toLowerCase().includes(searchQuery.toLowerCase()));
  // --- Context menu logic (merge from below) ---
  $: if (active && !alreadyLoaded) {
    fetchFunctionList();
    alreadyLoaded = true;
  }
  $: if (!active) {
    alreadyLoaded = false;
  }

  // Refresh function list on global signal (from PanelMain or anywhere)
  $: if ($functionsRefreshKey !== undefined) {
    fetchFunctionList();
  }

  function handleContextMenu(e: MouseEvent, idx: number) {
    e.preventDefault();
    showMenu = true;
    menuX = e.clientX;
    menuY = e.clientY;
    // Parse name and index out for action
    const [name, idxStr] = splitText(filteredList[idx]);
    selectedFunctionName = name + idxStr;
    selectedFunctionIdx = parseInt(idxStr.replace("@", ""));
  }
  async function copyFnMenu() {
    if (selectedFunctionName) {
      await navigator.clipboard.writeText(selectedFunctionName);
    }
    showMenu = false;
  }
  function editFnMenu() {
    // Cross-panel signal: set the functionToEdit writable store for PanelMain
    if (selectedFunctionIdx !== null) {
      functionToEdit.set(selectedFunctionIdx);
      mainPanelTab.set("constructor");
    }
    showMenu = false;
  }
  async function deleteFnMenu() {
    if (selectedFunctionIdx == null) return;
    // Only ask the user BEFORE running any deletion
    let doDelete = await new Promise<boolean>((resolve) => {
      // Use a setTimeout to guarantee the event bubble and menu closes before dialog
      setTimeout(() => resolve(confirm("Delete this function?")), 0);
    });
    if (!doDelete) { showMenu = false; return; }
    await invoke("delete_function", { index: selectedFunctionIdx });
    await fetchFunctionList();
    showMenu = false;
  }
  // Dismiss menu on global click/scroll/resize
  if (typeof window !== 'undefined') {
    window.addEventListener('click', () => showMenu = false);
    window.addEventListener('scroll', () => showMenu = false, true);
    window.addEventListener('resize', () => showMenu = false, true);
  }
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
          aria-label="Function Context Menu"
          on:contextmenu={(e) => handleContextMenu(e, index)}
          on:keydown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              const target = e.currentTarget as HTMLElement;
              const rect = target.getBoundingClientRect();
              showMenu = true;
              menuX = rect.left;
              menuY = rect.top + rect.height;
              const [name, idxStr] = splitText(filteredList[index]);
              selectedFunctionName = name + idxStr;
              selectedFunctionIdx = parseInt(idxStr.replace("@", ""));
              e.preventDefault();
            }
          }}
        >
          <button on:click={onClickButton} type="button" class="btn w-full preset-filled-surface-500 text-xs truncate">
            <div class="flex flex-row justify-start w-full">
              <span id="functionName" class="pointer-events-none">{splitText(filteredList[index])[0]}</span>
              <span id="functionIndex" class="text-tertiary-50 pointer-events-none">{splitText(filteredList[index])[1]}</span>
            </div>
          </button>
        </div>
      </VirtualList>
  </div>
    {#if showMenu && selectedFunctionIdx !== null}
      <div
        class="fixed z-50 bg-surface-700 border border-surface-600 rounded shadow p-1"
        style="left: {menuX}px; top: {menuY}px; min-width: 120px"
        role="menu"
        aria-label="Function actions"
        tabindex="-1"
        on:click={() => showMenu = false}
        on:keydown={(e) => { if (e.key === "Escape") showMenu = false; }}
      >
        <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
                role="menuitem"
                tabindex="0"
                on:click|stopPropagation={copyFnMenu}
                on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') copyFnMenu(); }}>
          Copy
        </button>
        <button class="block w-full text-left p-1 hover:bg-primary-400/30 rounded"
                role="menuitem"
                tabindex="0"
                on:click|stopPropagation={editFnMenu}
                on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') editFnMenu(); }}>
          Edit
        </button>
        <button class="block w-full text-left p-1 hover:bg-error-400/30 text-error-400 rounded"
                role="menuitem"
                tabindex="0"
                on:click|stopPropagation={deleteFnMenu}
                on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') deleteFnMenu(); }}>
          Delete
        </button>
      </div>
    {/if}
</div>

<!-- FunctionConstructor editing modal is now controlled by parent via the custom event 'edit-function'. -->