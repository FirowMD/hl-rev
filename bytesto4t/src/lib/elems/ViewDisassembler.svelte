<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { save } from "@tauri-apps/plugin-dialog";
  import VirtualList from 'svelte-tiny-virtual-list';


  let disassemblerTitle = "Disassembler";
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
      functionName = functionName.split(" ")[1];
      var funcName = "disasm_" + functionName.split("@")[1] + ".txt";
      var findex = functionName.split("@")[1];
      findex = "@" + findex;

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

  let codeElement: HTMLPreElement;

  onMount(() => {
    window.addEventListener("bytecode-item-selected", bytecodeItemSelectedHandler);
    updateDisassembler();
  });
</script>

<div class="h-full overflow-y-auto">
  <div class="p-2 space-y-2 preset-outlined-surface-500 bg-surface-900 h-full">
    <header class="h-12">
      <div class="flex flex-row items-start justify-between">
        <h5 class="h5">
          {disassembledCode.split('\n')[0]}
        </h5>
        {#if functionName !== ""}
          <button type="button" class="btn preset-outlined-surface-500 bg-surface-800" onclick={onClickSaveDisasmHandler}>
            Save
          </button>
        {/if}
      </div>
    </header>
    <section class="p-2 h-[calc(100%-3rem)]">
        {#if disassembledCode.length > 80000}
          <VirtualList width="100%" height="100%" itemCount={disassembledCodeLineNumber - 1} itemSize={25} overscanCount={200}>
            <div slot="item" let:index let:style {style}>
              <div class="w-full text-left truncate">
                <span>{disassembledCodeOptimized[index + 1]}</span>
              </div>
            </div>
          </VirtualList>
        {:else}
          <div class="h-full bg-surface-800 overflow-y-auto p-4">
            <pre>{disassembledCode.split('\n').slice(1).join('\n')}</pre>
          </div>
        {/if}
    </section>
  </div>
</div>