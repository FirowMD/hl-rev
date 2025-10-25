<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open, message } from "@tauri-apps/plugin-dialog";
  import {
    functionsRefreshKey,
    typesRefreshKey,
    globalsRefreshKey,
    nativesRefreshKey,
    constantsRefreshKey,
    stringsRefreshKey,
    intsRefreshKey,
    floatsRefreshKey,
    bytesRefreshKey
  } from "../types";

  let selectedPath: string | null = $state(null);
  let merging: boolean = $state(false);
  let statusMsg: string = $state("");
  let postMergeInfo: string = $state("");

  async function pickAndMerge() {
    try {
      const result = await open({
        multiple: false,
        title: "Select bytecode to merge",
        filters: [
          { name: "HashLink Bytecode", extensions: ["hl", "dat"] },
          { name: "All Files", extensions: ["*"] }
        ]
      });

      if (!result) return;
      selectedPath = Array.isArray(result) ? result[0] : result;
      merging = true;
      statusMsg = "Merging...";

      await invoke("merge_bytecode_with_file", { filePath: selectedPath });

      // Trigger refresh for all relevant views
      functionsRefreshKey.update(k => k + 1);
      typesRefreshKey.update(k => k + 1);
      globalsRefreshKey.update(k => k + 1);
      nativesRefreshKey.update(k => k + 1);
      constantsRefreshKey.update(k => k + 1);
      stringsRefreshKey.update(k => k + 1);
      intsRefreshKey.update(k => k + 1);
      floatsRefreshKey.update(k => k + 1);
      bytesRefreshKey.update(k => k + 1);

      // Fetch dashboard info to summarize the result
      const info = await invoke<string>("get_dashboard_info");
      postMergeInfo = String(info);
      statusMsg = "Merge complete";
    } catch (error) {
      console.error("Merge failed:", error);
      statusMsg = "Merge failed";
      await message(`Failed to merge: ${error}`, { title: "Error", kind: "error" });
    } finally {
      merging = false;
    }
  }
</script>

<section class="space-y-3">
  <h5 class="h5">Merge Bytecode</h5>
  <div class="space-y-2">
    <button type="button" class="btn preset-filled-surface-500 w-full" onclick={pickAndMerge} disabled={merging}>
      <span>{merging ? "Merging..." : "Select .hl/.dat and Merge"}</span>
    </button>
    {#if selectedPath}
      <div class="text-sm text-surface-400">Selected: {selectedPath}</div>
    {/if}
    {#if statusMsg}
      <div class="text-sm {merging ? 'text-warning-500' : 'text-success-500'}">{statusMsg}</div>
    {/if}
    {#if postMergeInfo}
      <div class="p-2 bg-surface-800 rounded">
        <pre class="text-xs whitespace-pre-wrap">{postMergeInfo}</pre>
      </div>
    {/if}
  </div>
</section>