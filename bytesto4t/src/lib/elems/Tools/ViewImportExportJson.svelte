<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save, open } from "@tauri-apps/plugin-dialog";

  let enteredValue: number = $state(0);

  async function onClickImportTypeJson() {
    try {
      const result = await open({
        multiple: false,
        filters: [
          {
            name: "JSON Files",
            extensions: ["json"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("import_type_json", { jsonPath: result });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }

  async function onClickExportTypeJson() {
    try {
      const result = await save({
        defaultPath: "type.json",
        title: "Export type",
        filters: [
          {
            name: "JSON Files",
            extensions: ["json"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("export_type_json", { typeIndex: enteredValue.toString(), filePath: result });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }

  async function onClickImportFunctionJson() {
    try {
      const result = await open({
        multiple: false,
        filters: [
          {
            name: "JSON Files",
            extensions: ["json"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("import_function_json", { jsonPath: result });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }

  async function onClickExportFunctionJson() {
    try {
      const result = await save({
        defaultPath: "function.json",
        title: "Export function",
        filters: [
          {
            name: "function.json",
            extensions: ["json"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("export_function_json", { functionIndex: "@" + enteredValue.toString(), filePath: result });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }
</script>

<section class="space-y-2">
  <div class="flex justify-between items-center">
    <h5 class="h5">Import/Export JSON</h5>
  </div>
  <div class="space-y-2">
    <span>Enter index of element:</span>
    <input
        type="number"
        class="input bg-surface-800 flex-1 focus:outline-none"
        placeholder="Enter index of element"
        oninput={(e) => {
          const value = parseInt(e.currentTarget.value);
          if (!isNaN(value) && value >= 0) {
            console.log("Value:", value);
            enteredValue = value;
          }
        }}
    />
    {#if enteredValue !== null}
      <div class="text-sm text-secondary-500">
        Selected index: {enteredValue}
      </div>
    {/if}
    <div class="space-y-2">
      <div class="flex flex-row space-x-2">
        <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickImportFunctionJson}>
          <span>Import Function</span>
        </button>
        <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickImportTypeJson}>
          <span>Import Class</span>
        </button>
      </div>
        <div class="flex flex-row space-x-2">
        <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickExportFunctionJson}>
          <span>Export Function</span>
        </button>
        <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickExportTypeJson}>
          <span>Export Class</span>
        </button>
      </div>
    </div>
  </div>
</section>