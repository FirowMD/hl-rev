<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { open, message } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { BaseDirectory, readTextFile, writeFile } from "@tauri-apps/plugin-fs";

  interface FileStatus {
    name: string;
    lines: number;
  }
  
  let addressesStatus: FileStatus | null = $state(null);
  let filteredStatus: FileStatus | null = $state(null);
  let recognizedPreview: string = $state("");

  async function updateRecognizedPreview() {
    if (!loadedContent) return;

    try {
      const addresses: string[] = await invoke("get_function_addresses");
      if (addresses.length === 0) return;
      
      const functionList: string[] = await invoke("get_function_list");

      const previewLines = loadedContent.split('\n')
        .map(line => {
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

      recognizedPreview = previewLines.join('\n');
    } catch (error) {
      console.error('Failed to update preview:', error);
    }
  }

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
        const content = await readTextFile(funcPath);
        addressesStatus = {
          name: funcPath.split(/[/\\]/).pop() || "",
          lines: content.split('\n').length
        };
      }
    } catch (error) {
      await message(
        `Failed to load function addresses: ${error}`, 
        { title: "Error", kind: "error" }
      );
    }
  }

  async function onClickLoadFilteredHandler() {
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
      
      // Store the loaded content for later use
      loadedContent = await readTextFile(inputPath);
      filteredStatus = {
        name: inputPath.split(/[/\\]/).pop() || "",
        lines: loadedContent.split('\n').length
      };
      await updateRecognizedPreview();
      
    } catch (error) {
      await message(
        `Failed to load filtered content: ${error}`,
        { title: "Error", kind: "error" }
      );
    }
  }
  
  async function onClickSaveRecognizedHandler() {
    try {
      if (!loadedContent) {
        await message("No filtered content loaded", { title: "Error", kind: "error" });
        return;
      }

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

      const outputLines = loadedContent.split('\n').map(line => {
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
    } catch (error) {
      await message(
        `Failed to recognize functions: ${error}`,
        { title: "Error", kind: "error" }
      );
    }
  }

  let loadedContent: string | null = null;

  onMount(() => {
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="card p-2 space-y-2 variant-soft-secondary">
    <section class="card p-4 variant-soft-secondary space-y-2">
      <h4 class="h4">Export</h4>
      <div class="space-y-2">
        <button type="button" class="btn variant-soft-secondary w-full" onclick={onClickExportFunctionsHandler}>
          <span>Functions</span>
        </button>
        <button type="button" class="btn variant-soft-secondary w-full" onclick={onClickExportTypesHandler}>
          <span>Classes</span>
        </button>
        <button type="button" class="btn variant-soft-secondary w-full" onclick={onClickExportFilesHandler}>
          <span>Files</span>
        </button>
      </div>
    </section>
    <section class="card p-4 variant-soft-secondary space-y-2">
      <h4 class="h4">Function recognizer</h4>
      <div class="flex flex-row space-x-2">
        <button type="button" class="btn variant-soft-secondary w-full" onclick={onClickLoadFunctionAddressesHandler}>
          <span>Load addresses</span>
        </button>
        <button type="button" class="btn variant-soft-secondary w-full" onclick={onClickLoadFilteredHandler}>
          <span>Load filtered</span>
        </button>
        <button type="button" class="btn variant-soft-secondary w-full" onclick={onClickSaveRecognizedHandler}>
          <span>Save recognized</span>
        </button>
      </div>
      <div class="flex flex-col space-y-2 mt-2">
        <div class="flex flex-row space-x-4">
          <label class="flex items-center space-x-2">
            <input 
              type="checkbox" 
              class="checkbox variant-soft-primary" 
              checked={!!addressesStatus}
              disabled
            />
            <span class="text-sm">
              {addressesStatus ? 
                `Addresses: ${addressesStatus.name} (${addressesStatus.lines} lines)` : 
                'No addresses loaded'}
            </span>
          </label>
          <label class="flex items-center space-x-2">
            <input 
              type="checkbox" 
              class="checkbox variant-soft-primary" 
              checked={!!filteredStatus}
              disabled
            />
            <span class="text-sm">
              {filteredStatus ? 
                `Filtered: ${filteredStatus.name} (${filteredStatus.lines} lines)` : 
                'No filtered content loaded'}
            </span>
          </label>
        </div>
        {#if recognizedPreview}
          <div class="card p-2 variant-soft-primary">
            <pre class="text-sm font-mono whitespace-pre-wrap h-96 overflow-y-auto">{recognizedPreview}</pre>
          </div>
        {/if}
      </div>
    </section>
  </div>
</div>