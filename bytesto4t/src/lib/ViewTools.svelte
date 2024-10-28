<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";

  async function onClickExportFunctionsHandler() {
    try {
      const result = await save({
        defaultPath: "functions.txt",
        title: "Export functions",
        filters: [
          {
            name: "functions.txt",
            extensions: ["txt"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("save_function_list", { filePath: result });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }

  async function onClickExportTypesHandler() {
    try {
      const result = await save({
        defaultPath: "classes.txt",
        title: "Export classes",
        filters: [
          {
            name: "classes.txt",
            extensions: ["txt"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("save_type_list", { filePath: result });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }

  async function onClickExportFilesHandler() {
    try {
      const result = await save({
        defaultPath: "files.txt",
        title: "Export files",
        filters: [
          {
            name: "files.txt",
            extensions: ["txt"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("save_file_list", { filePath: result });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }

  onMount(() => {
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="card p-2 space-y-2 variant-soft-secondary">
    <section class="card p-4 variant-soft-secondary space-y-2">
      <h4 class="h4">Export</h4>
      <div class="space-y-2">
        <button type="button" class="btn variant-soft-secondary w-full" on:click={onClickExportFunctionsHandler}>
          <span>Functions</span>
        </button>
        <button type="button" class="btn variant-soft-secondary w-full" on:click={onClickExportTypesHandler}>
          <span>Classes</span>
        </button>
        <button type="button" class="btn variant-soft-secondary w-full" on:click={onClickExportFilesHandler}>
          <span>Files</span>
        </button>
      </div>
    </section>
  </div>
</div>