<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { save, message } from "@tauri-apps/plugin-dialog";
  import { writeFile } from "@tauri-apps/plugin-fs";
  import { onMount } from "svelte";

  interface SelectedItem {
    name: string;
    type: string;
  }

  let selectedItem: string | null = $state(null);

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

  async function generatePattern() {
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
  }

  onMount(() => {
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
        onclick={generatePattern}
      >
        Generate Pattern
      </button>
    </div>
    <div class="text-sm">
      Select a class/type and click "Generate Pattern" to create an ImHex pattern file.
    </div>
  </div>
</section>