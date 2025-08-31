<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, message } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  let chosenRecentFile: string = "";
  let targetFilePath: string = "";
  let recentFiles: string[] = [];
  let contextMenuVisible: boolean = false;
  let contextMenuX: number = 0;
  let contextMenuY: number = 0;
  let contextMenuFile: string = "";

  async function setTargetFilePath(filePath: string) {
    try {
      await invoke("set_target_file_path", { filePath: filePath });
    } catch (error) {
      throw error;
    }
  }

  async function onClickBrowseHandler() {
    try {
      const result = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: "hlboot.dat",
            extensions: ["dat"]
          },
          {
            name: "output.hl",
            extensions: ["hl"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        targetFilePath = result;
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }

  async function onChangeListBoxItemHandler(event: Event) {
    targetFilePath = chosenRecentFile;
  }

  async function switchOnNextPage() {
    try {
      if (targetFilePath !== "") {
        await setTargetFilePath(targetFilePath);
        await addRecentFile();
        window.location.href = "/main";
      }
    } catch (error) {
      await message("" + error, { title: 'Error', kind: 'error' });
    }
  }

  async function onEnterDown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      await switchOnNextPage();
    }
  }

  async function addRecentFile() {
    try {
      await invoke("add_config_recent_file", { filePath: targetFilePath });
      await invoke("save_config");
    } catch (error) {
      console.error("Error adding recent file:", error);
    }
  }

  async function getRecentFiles() {
    try {
      const response = await invoke("get_config_recent_files") as string[];
      recentFiles = response;
    } catch (error) {
      console.error("Error getting recent files:", error);
    }
  }

  async function removeRecentFile(filePath: string) {
    try {
      await invoke("remove_config_recent_file", { filePath: filePath });
      await invoke("save_config");
      await getRecentFiles();
      contextMenuVisible = false;
    } catch (error) {
      console.error("Error removing recent file:", error);
    }
  }

  function showContextMenu(event: MouseEvent, filePath: string) {
    event.preventDefault();
    contextMenuFile = filePath;
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    contextMenuVisible = true;
  }

  function hideContextMenu() {
    contextMenuVisible = false;
  }

  function handleGlobalClick(event: MouseEvent) {
    if (contextMenuVisible && !(event.target as Element)?.closest('.context-menu')) {
      hideContextMenu();
    }
  }

  onMount(async () => {
    await invoke("init_config", {});
    await getRecentFiles();
    document.addEventListener('click', handleGlobalClick);
    
    return () => {
      document.removeEventListener('click', handleGlobalClick);
    };
  });
</script>

<main class="w-full h-full p-4 bg-gradient-to-b from-surface-950 to-surface-950">
  <div class="flex flex-col gap-2 w-full h-full">
    <div class="flex flex-row items-end space-x-4">
      <div class="h-14 w-14">
        <img class="h-auto max-w-full rounded-lg" src="/assets/logo_v2.png" alt="Logo">
      </div>
      <h1
        class="text-2xl font-bold text-surface-500"
      >
        ByteSto4t v2.1
      </h1>
    </div>
    <div class="flex flex-row gap-2 justify-between w-full h-8">
      <input
        class="input w-full focus:outline-none"
        type="text"
        placeholder="Input"
        bind:value={targetFilePath}
        onkeydown={onEnterDown}
      />
      <button
        type="button"
        class="btn preset-filled-surface-500 w-fit"
        onclick={onClickBrowseHandler}
      >
        Browse
      </button>
    </div>
    <button
      type="button"
      class="btn preset-filled-surface-500 w-full"
      onclick={switchOnNextPage}
    >
      Ready!
    </button>
    <form class="w-full h-full">
      <select 
        class="select w-full h-full focus:outline-none overflow-y-auto" 
        size="5" 
        bind:value={chosenRecentFile}
        onchange={onChangeListBoxItemHandler}
      >
        {#each recentFiles as recentFile}
          <option 
            value={recentFile}
            oncontextmenu={(e) => showContextMenu(e, recentFile)}
          >
            {recentFile}
          </option>
        {/each}
      </select>
    </form>
  </div>
  
  {#if contextMenuVisible}
    <div 
      class="context-menu fixed z-50 bg-surface-800 border border-surface-600 rounded-lg shadow-lg py-1 min-w-48"
      style="left: {contextMenuX}px; top: {contextMenuY}px;"
    >
      <button
        class="w-full text-left px-4 py-2 hover:bg-surface-700 transition-colors text-surface-100"
        onclick={() => removeRecentFile(contextMenuFile)}
      >
        Remove from list
      </button>
    </div>
  {/if}
</main>
