<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { open, message } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { BaseDirectory, readTextFile, writeFile } from "@tauri-apps/plugin-fs";

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

  async function onClickLoadFunctionAddressesHandler() {
    try {
      const funcPath = await open({
        multiple: false,
        filters: [{
          name: 'Text Files',
          extensions: ['txt']
        },
        {
          name: 'All Files',
          extensions: ['*']
        }]
      });

      if (funcPath) {
        await invoke("load_function_addresses_from_file", { filePath: funcPath });
        await message("Function addresses loaded successfully", { title: "Success" });
      }
    } catch (error) {
      await message(
        `Failed to load function addresses: ${error}`, 
        { title: "Error", kind: "error" }
      );
    }
  }

  async function onClickRecognizeFunctionsHandler() {
    try {
      const inputPath = await open({
        multiple: false,
        filters: [{
          name: 'Text Files',
          extensions: ['txt']
        },
        {
          name: 'All Files',
          extensions: ['*']
        }]
      });

      if (!inputPath) return;

      const addresses: string[] = await invoke("get_function_addresses");
      if (addresses.length === 0) {
        await message("No addresses loaded", { title: "Error", kind: "error" });
        return;
      }
      
      const functionList: string[] = await invoke("get_function_list");

      const outputPath = await save({
        defaultPath: "recognized.txt",
        title: "Save recognized functions",
        filters: [{
          name: "Text Files",
          extensions: ["txt"]
        },
        {
          name: "All Files",
          extensions: ["*"]
        }]
      });

      if (!outputPath) return;

      const inputContent = await readTextFile(inputPath);

      const outputLines = inputContent.split('\n').map(line => {
        var findex = addresses.findIndex(addr => addr === line.trim());
        if (findex !== -1) {
          var func = "";
          for (var i = 0; i < functionList.length; i++) {
            var funcIndex = parseInt(functionList[i].split('@')[1]);
            if (funcIndex === findex) {
              func = functionList[i];
              break;
            }
          }
          return `${line.trim()} ${func}`;
        }
        return line;
      });

      await writeFile(outputPath, new TextEncoder().encode(outputLines.join('\n')), { baseDir: BaseDirectory.Home });

      await message("Functions recognized successfully", { title: "Success" });
    } catch (error) {
      await message(
        `Failed to recognize functions: ${error}`,
        { title: "Error", kind: "error" }
      );
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
    <section class="card p-4 variant-soft-secondary space-y-2">
      <h4 class="h4">Function recognizer</h4>
      <div class="space-y-2">
        <button type="button" class="btn variant-soft-secondary w-full" on:click={onClickLoadFunctionAddressesHandler}>
          <span>Load addresses</span>
        </button>
        <button type="button" class="btn variant-soft-secondary w-full" on:click={onClickRecognizeFunctionsHandler}>
          <span>Recognize functions</span>
        </button>
      </div>
    </section>
  </div>
</div>