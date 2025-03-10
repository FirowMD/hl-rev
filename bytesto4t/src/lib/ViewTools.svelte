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
        // Read the content and convert it to a string
        const content = await readTextFile(funcPath);
        const contentStr = typeof content === 'string' ? content : new TextDecoder().decode(content);
        
        // Load the addresses into the backend
        await invoke("load_function_addresses_from_file", { filePath: funcPath });
        
        // Update the status display
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

  async function onClickFindReferencesHandler() {
    if (elementIndex === null) {
      elementIndex = null;
      references = [];
      await invoke("clear_references");
      return;
    }

    try {
      references = await invoke("get_all_references", { elemIdx: elementIndex });

      if (references.length === 0) {
        await message(`No references found for element ${elementIndex}`, { title: "Info", kind: "info" });
      }
    } catch (error) {
      await message(
        `Failed to find references: ${error}`,
        { title: "Error", kind: "error" }
      );
    }
  }

  async function onClickReference(ref: string) {
    const [funcPart] = ref.split('###');
    const [name, id, findex] = funcPart.split('@');
    const fullName = `${name}@${id}@${findex}`;

    console.log("findex: `" + findex + "`");
    await invoke("set_selected_item", {
      appItem: {
        index: findex,
        typ: "function"
      }
    });

    console.log("fullName: `" + fullName + "`");
    await invoke("add_history_item", {
      item: {
        name: fullName,
        typ: "function",
        timestamp: new Date().toISOString()
      }
    });

    console.log("fullName: `" + fullName + "`");
    const ev = new CustomEvent("bytecode-item-selected", {
      detail: {
        name: fullName,
        type: "function"
      }
    });

    window.dispatchEvent(ev);
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

  async function onClickSaveReferencesHandler() {
    try {
      if (references.length === 0) {
        await message("No references to save", { title: "Error", kind: "error" });
        return;
      }

      const result = await save({
        defaultPath: `references_${elementIndex}.csv`,
        title: "Save references",
        filters: [{
          name: "CSV Files",
          extensions: ["csv"]
        },
        {
          name: "All Files",
          extensions: ["*"]
        }]
      });

      if (result) {
        const csvContent = references.map(ref => {
          const { funcPart, pos, op } = parseReference(ref);
          return `${funcPart},${pos},${op}`;
        }).join('\n');

        await writeFile(result, new TextEncoder().encode(csvContent));
      }
    } catch (error) {
      await message(
        `Failed to save references: ${error}`,
        { title: "Error", kind: "error" }
      );
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
    <section class="card p-4 variant-soft-secondary space-y-2">
      <div class="flex justify-between items-center">
        <h4 class="h4">Reference finder</h4>
        {#if references.length > 0}
          <button 
            type="button" 
            class="btn variant-soft-secondary" 
            onclick={onClickSaveReferencesHandler}
          >
            Save to csv
          </button>
        {/if}
      </div>
      <div class="flex flex-row space-x-2">
        <input 
          type="number" 
          class="input variant-form-material" 
          placeholder="Element index"
          bind:value={elementIndex}
        />
        <button 
          type="button" 
          class="btn variant-soft-secondary" 
          onclick={onClickFindReferencesHandler}
        >
          Find
        </button>
      </div>
      {#if references.length > 0}
        <div class="card p-2 variant-soft-secondary">
          <VirtualList
            itemCount={references.length}
            itemSize={35}
            height={400}
            width="100%"
          >
            <div slot="item" let:index let:style {style}>
              {@const { funcPart, pos, op } = parseReference(references[index])}
              <button 
                class="grid grid-cols-3 gap-4 p-2 hover:bg-secondary-700/20 w-full text-left"
                onclick={() => onClickReference(references[index])}
              >
                <div class="truncate">{funcPart}</div>
                <div>{pos}</div>
                <div>{op}</div>
              </button>
            </div>
          </VirtualList>
        </div>
      {/if}
    </section>
    <section class="card p-4 variant-soft-secondary space-y-2">
      <div class="flex justify-between items-center">
        <h4 class="h4">ImHex Pattern Generator</h4>
      </div>
      <div class="space-y-2">
        <div class="flex flex-row space-x-2">
          <input 
            type="text" 
            class="input variant-form-material flex-1" 
            placeholder="No class/type selected"
            value={selectedItem ?? ""}
            disabled
          />
          <button 
            type="button" 
            class="btn variant-soft-secondary" 
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
        <div class="text-sm text-secondary-400">
          Select a class/type and click "Generate Pattern" to create an ImHex pattern file.
        </div>
      </div>
    </section>
  </div>
</div>