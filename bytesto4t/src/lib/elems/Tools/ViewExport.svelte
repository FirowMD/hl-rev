<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";

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

  async function onClickExportStrippedHandler() {
    try {
      const result = await save({
        defaultPath: "stripped.hl",
        title: "Export stripped bytecode",
        filters: [
          {
            name: "stripped.hl",
            extensions: ["hl"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("save_stripped_bytecode", { filePath: result });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }
</script>

<section class="space-y-2">
  <h5 class="h5">Export</h5>
  <div class="space-y-2">
    <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickExportFunctionsHandler}>
      <span>Functions</span>
    </button>
    <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickExportTypesHandler}>
      <span>Classes</span>
    </button>
    <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickExportFilesHandler}>
      <span>Files</span>
    </button>
    <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickExportStrippedHandler}>
      <span>Stripped Bytecode</span>
    </button>
  </div>
</section>