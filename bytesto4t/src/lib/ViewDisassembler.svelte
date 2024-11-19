<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { CodeBlock } from '@skeletonlabs/skeleton';
  import { save } from "@tauri-apps/plugin-dialog";
  import hljs from 'highlight.js/lib/core';
  import 'highlight.js/styles/github-dark.css';
  import { storeHighlightJs } from '@skeletonlabs/skeleton';
  import VirtualList from 'svelte-tiny-virtual-list';

  storeHighlightJs.set(hljs);
  hljs.registerLanguage('hl', function(hljs) {
    return {
      contains: [
        {
          className: 'keyword',
          begin: /\b(?:Mov|Int|Float|Bool|Bytes|String|Null|Add|Sub|Mul|SDiv|UDiv|SMod|UMod|Shl|SShr|UShr|And|Or|Xor|Neg|Not|Incr|Decr|CallCall1|Call2|Call3|Call4|CallN|CallMethod|CallThis|CallClosure|StaticClosure|InstanceClosure|VirtualClosure|GetGlobal|SetGlobal|Field|SetField|GetThis|SetThis|DynGet|DynSet|JTrue|JFalse|JNull|JNotNull|JSLt|JSGte|JSGt|JSLte|JULt|JUGte|JNotLt|JNotGte|JEq|JNotEq|JAlways|ToDyn|ToSFloat|ToUFloat|ToInt|SafeCast|UnsafeCast|ToVirtual|Label|Ret|Throw|Rethrow|Switch|NullCheck|Trap|EndTrap|GetI8|GetI16|GetMem|GetArray|SetI8|SetI16|SetMem|SetArray|New|ArraySize|Type|GetType|GetTID|Ref|Unref|Setref|MakeEnum|EnumAlloc|EnumIndex|EnumField|SetEnumField|Assert|RefData|RefOffset|Nop|Prefetch|Asm)\b/
        },
        {
          className: 'number',
          begin: /\b\d+\b/
        },
        {
          className: 'type',
          begin: /\b(?:hl\.types\.\w+|void|i8|i16|i32|i64|f32|f64|bool|type|dynamic|hl\.BaseType|array|hl\.Class|String|bytes)\b/
        }
      ]
    };
  });

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

<div class="h-full overflow-y-auto">
  <div class="card p-2 space-y-2 variant-soft-secondary h-full">
    <header class="card-header h-12">
      <div class="flex flex-row items-start justify-between">
        <h3 class="h3">
          {disassemblerTitle}
        </h3>
        {#if functionName !== ""}
          <button type="button" class="btn variant-soft-secondary" on:click={onClickSaveDisasmHandler}>
            Save
          </button>
        {/if}
      </div>
    </header>
    <section class="p-2 h-[calc(100%-3rem)]">

        {#if disassembledCode.length > 80000}
          <VirtualList width="100%" height="100%" itemCount={disassembledCodeLineNumber} itemSize={25} overscanCount={200}>
            <div slot="item" let:index let:style {style}>
              <div class="w-full text-left truncate">
                <span>{disassembledCodeOptimized[index]}</span>
              </div>
            </div>
          </VirtualList>
        {:else}
          <CodeBlock class="h-full overflow-y-auto" language="hl" background="bg-secondary-900" lineNumbers={true} code={disassembledCode} />
        {/if}
    </section>
  </div>
</div>