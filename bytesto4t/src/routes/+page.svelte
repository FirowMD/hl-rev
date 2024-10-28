<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, message } from "@tauri-apps/plugin-dialog";
  import { ListBox, ListBoxItem } from "@skeletonlabs/skeleton";
  import { onMount } from "svelte";

  let chosenRecentFile: string = ""; //! test
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

  async function getColorscheme() {
    try {
      const response = await invoke("get_config_colorscheme") as string;
      document.body.setAttribute("data-theme", response);
    } catch (error) {
      console.error("Error getting colorscheme:", error);
    }
  }

  async function getTheme() {
    try {
      const response = await invoke("get_config_theme") as string;
      if (response === "dark") {
        document.documentElement.classList.add("dark");
        document.documentElement.classList.remove("light");
      } else {
        document.documentElement.classList.add("light");
        document.documentElement.classList.remove("dark");
      }
    } catch (error) {
      console.error("Error getting theme:", error);
    }
  }

  onMount(async () => {
    await invoke("init_config", {});
    await getRecentFiles();
    await getColorscheme();
    await getTheme();
  });
</script>

<div class="flex flex-col p-4 space-y-4 h-full">
  <div class="flex flex-row items-end space-x-4">
    <div class="h-16 w-16">
      <img class="h-auto max-w-full rounded-lg" src="/assets/logo.png" alt="Logo">
    </div>
    <h1 class="text-4xl">ByteSto4t v.1</h1>
  </div>
  <div class="browse-field flex space-x-4">
    <input class="input" type="text" placeholder="Enter path to file" bind:value={targetFilePath} on:keydown={onEnterDown} />
    <button type="button" class="btn variant-soft-primary" on:click={onClickBrowseHandler}>Browse</button>
  </div>
  {#if targetFilePath !== ""}
    <button type="button" class="btn variant-filled-primary" on:click={switchOnNextPage}>Ready!</button>
  {:else}
    <button type="button" class="btn variant-filled-primary" disabled>Ready!</button>
  {/if}
  <div class="card overflow-y-auto h-full">
    <ListBox active="variant-soft-primary h-full">
      {#each recentFiles as file}
        <ListBoxItem on:change={onChangeListBoxItemHandler} bind:group={chosenRecentFile} name="medium" value={file}>{file}</ListBoxItem>
      {/each}
    </ListBox>
  </div>
</div>