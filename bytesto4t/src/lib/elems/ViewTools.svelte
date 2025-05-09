<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save } from "@tauri-apps/plugin-dialog";
  import { open, message } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import { BaseDirectory, readTextFile, writeFile } from "@tauri-apps/plugin-fs";
  import VirtualList from 'svelte-tiny-virtual-list';

  interface FileStatus {
    name: string;
    lines: number;
  }
  
  let addressesStatus: FileStatus | null = $state(null);
  let filteredStatus: FileStatus | null = $state(null);
  let recognizedPreview: string = $state("");
  let elementIndex: number | null = $state(null);
  let references: string[] = $state([]);
  let selectedItem: string | null = $state(null);

  interface SelectedItem {
    name: string;
    type: string;
  }

  interface ImHexPattern {
    pattern: string;
  }

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
        const content = await readTextFile(funcPath);
        const contentStr = typeof content === 'string' ? content : new TextDecoder().decode(content);
        
        await invoke("load_function_addresses_from_file", { filePath: funcPath });
        
        addressesStatus = {
          name: funcPath.split(/[/\\]/).pop() || "",
          lines: contentStr.split('\n').length
        };
      }
    } catch (error) {
      console.error("Full error:", error);
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
      const content = await readTextFile(inputPath);
      loadedContent = typeof content === 'string' ? content : new TextDecoder().decode(content);
      
      filteredStatus = {
        name: inputPath.split(/[/\\]/).pop() || "",
        lines: loadedContent.split('\n').length
      };
      
      await updateRecognizedPreview();
      
    } catch (error) {
      console.error("Full error:", error);
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

  function parseReference(ref: string) {
    const [funcPart, pos, op] = ref.split('###');
    return { funcPart, pos, op };
  }

  let loadedContent: string | null = null;

  async function loadSavedReferences() {
    try {
      const saved = await invoke<[number, string[]] | null>("get_saved_references");
      if (saved) {
        const [idx, refs] = saved;
        elementIndex = idx;
        references = refs;
      }
    } catch (error) {
      console.error("Failed to load saved references:", error);
    }
  }

  async function updateSelectedItem() {
    try {
      const item = await invoke<SelectedItem | null>("get_selected_item");
      if (item) {
        selectedItem = item.name;
      } else {
        selectedItem = null;
      }
    } catch (error) {
      console.error("Failed to get selected item:", error);
      selectedItem = null;
    }
  }

  onMount(() => {
    loadSavedReferences();
    window.addEventListener("bytecode-item-selected", async (e: Event) => {
      const ev = e as CustomEvent<{name: string, type: string}>;
      if (ev.detail.type === "class") {
        selectedItem = ev.detail.name;
      } else {
        selectedItem = null;
      }
    });
    
    updateSelectedItem();
  });
</script>

<div class="h-full preset-outlined-surface-500 bg-surface-900 overflow-y-auto">
  <div class="p-2 space-y-2">
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
      </div>
    </section>
    <section class="space-y-2">
      <h5 class="h5">Function recognizer</h5>
      <div class="flex flex-row space-x-2">
        <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickLoadFunctionAddressesHandler}>
          <span>Load addresses</span>
        </button>
        <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickLoadFilteredHandler}>
          <span>Load filtered</span>
        </button>
        <button type="button" class="btn preset-filled-surface-500 w-full" onclick={onClickSaveRecognizedHandler}>
          <span>Save recognized</span>
        </button>
      </div>
      <div class="flex flex-col space-y-2 mt-2">
        <div class="flex flex-row space-x-4">
          <div class="flex items-center space-x-2">
            <span class={`badge-icon ${addressesStatus ? 'preset-filled-success-500' : 'preset-filled-surface-500'}`}>
              {addressesStatus ? '✓' : '×'}
            </span>
            <span class="text-sm">
              {addressesStatus ? 
                `Addresses: ${addressesStatus.name} (${addressesStatus.lines} lines)` : 
                'No addresses loaded'}
            </span>
          </div>
          <div class="flex items-center space-x-2">
            <span class={`badge-icon ${filteredStatus ? 'preset-filled-success-500' : 'preset-filled-surface-500'}`}>
              {filteredStatus ? '✓' : '×'}
            </span>
            <span class="text-sm">
              {filteredStatus ? 
                `Filtered: ${filteredStatus.name} (${filteredStatus.lines} lines)` : 
                'No filtered content loaded'}
            </span>
          </div>
        </div>
        {#if recognizedPreview}
          <div class="p-2 bg-surface-800">
            <pre class="text-sm font-mono whitespace-pre-wrap h-96 overflow-y-auto">{recognizedPreview}</pre>
          </div>
        {/if}
      </div>
    </section>
    <section class="space-y-2">
      <div class="flex justify-between items-center">
        <h5 class="h5">ImHex Pattern Generator</h5>
      </div>
      <div class="space-y-2">
        <div class="flex flex-row space-x-2">
          <input 
            type="text" 
            class="input bg-surface-800 flex-1 focus:outline-none" 
            placeholder="No class selected"
            value={selectedItem ?? ""}
            disabled
          />
          <button 
            type="button" 
            class="btn preset-filled-surface-500" 
            disabled={!selectedItem}
            onclick={async () => {
              try {
                const pattern = await invoke<string>("generate_imhex_pattern");
                if (pattern) {
                  const result = await save({
                    defaultPath: "pattern.hexpat",
                    title: "Save ImHex pattern",
                    filters: [{
                      name: "ImHex Pattern",
                      extensions: ["hexpat"]
                    },
                    {
                      name: "All Files",
                      extensions: ["*"]
                    }]
                  });

                  if (result) {
                    await writeFile(result, new TextEncoder().encode(pattern));
                    await message("Pattern saved successfully!", { title: "Success", kind: "info" });
                  }
                }
              } catch (error) {
                await message(
                  `Failed to generate pattern: ${error}`,
                  { title: "Error", kind: "error" }
                );
              }
            }}
          >
            Generate Pattern
          </button>
        </div>
        <div class="text-sm">
          Select a class/type and click "Generate Pattern" to create an ImHex pattern file.
        </div>
      </div>
    </section>
  </div>
</div>