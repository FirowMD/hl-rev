<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import VirtualList from 'svelte-tiny-virtual-list';

  let functionName = "";
  let disassembledCode = "";
  let disassembledCodeOptimized: string[] = [];
  let disassembledCodeLineNumber = 0;

  async function bytecodeItemSelectedHandler(e: Event) {
    try {
      const ev = e as CustomEvent<{name: string, type: string}>;
      const itemType = ev.detail.type;

      if (itemType === "function" || itemType === "class") {
        functionName = ev.detail.name;
        updateDisassembler();
      }
    } catch (error) {
      console.log("Error fetching disassembled info:", error);
    }
  }

  async function onClickSaveDisasmHandler() {
    try {
      const selectedFunction = functionName.split(" ")[1] ?? functionName;
      const functionIndex = selectedFunction.split("@")[1];
      if (!functionIndex) return;

      const funcName = "disasm_" + functionIndex + ".txt";
      const findex = "@" + functionIndex;

      const result = await save({
        defaultPath: funcName,
        title: "Save disassembled code",
        filters: [
          {
            name: "disasm.txt",
            extensions: ["txt"]
          },
          {
            name: "All Files",
            extensions: ["*"]
          }
        ]
      });

      if (result) {
        await invoke("save_disassembled_code", { filePath: result, functionIndex: findex });
      }
    } catch (error) {
      console.error("Error opening file:", error);
    }
  }

  async function updateDisassembler() {
    const response = await invoke("get_disassembler_info") as string;
    if (response !== null) {
      functionName = response.split("\n")[0];
      disassembledCode = response;
      disassembledCodeOptimized = disassembledCode.split("\n");
      disassembledCodeLineNumber = disassembledCodeOptimized.length;
    }
  }

  onMount(() => {
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    updateDisassembler();
  });
</script>

<div class="h-full min-h-0 overflow-hidden">
  <section class="code-view flex h-full min-h-0 flex-col overflow-hidden rounded-sm">
    <header class="code-view-toolbar flex min-h-10 shrink-0 items-center justify-between gap-3 px-3 py-2">
      <div class="min-w-0 text-xs">
        {#if functionName !== ""}
          <span class="block truncate font-semibold text-surface-100" title={functionName}>{functionName}</span>
        {:else}
          <span class="text-surface-500">No function selected</span>
        {/if}
      </div>
      <button
        type="button"
        class="btn preset-tonal-grain-raised-surface"
        onclick={onClickSaveDisasmHandler}
        disabled={functionName === ""}
      >
        Save
      </button>
    </header>

    <div class="min-h-0 flex-1 overflow-hidden p-2">
      {#if disassembledCode.length > 80000}
        <VirtualList width="100%" height="100%" itemCount={disassembledCodeLineNumber} itemSize={25} overscanCount={200}>
          <div slot="item" let:index let:style {style}>
            <div class="w-full text-left truncate">
              <span>{disassembledCodeOptimized[index]}</span>
            </div>
          </div>
        </VirtualList>
      {:else}
        <div class="h-full bg-surface-800 overflow-y-auto p-4 rounded-container-token">
          <pre>{disassembledCode}</pre>
        </div>
      {/if}
    </div>
  </section>
</div>
