<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, message } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  let chosenRecentFile: string = "";
  let targetFilePath: string = "";
  let recentFiles: string[] = [];

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

  onMount(async () => {
    await invoke("init_config", {});
    await getRecentFiles();
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
        ByteSto4t v2.0
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
          <option value={recentFile}>{recentFile}</option>
        {/each}
      </select>
    </form>
  </div>
</main>
