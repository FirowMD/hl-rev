<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save, open, message } from "@tauri-apps/plugin-dialog";
  import { BaseDirectory, readTextFile, writeFile } from "@tauri-apps/plugin-fs";

  interface FileStatus {
    name: string;
    lines: number;
  }
  
  let addressesStatus: FileStatus | null = $state(null);
  let filteredStatus: FileStatus | null = $state(null);
  let recognizedPreview: string = $state("");
  let loadedContent: string | null = null;

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
</script>

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