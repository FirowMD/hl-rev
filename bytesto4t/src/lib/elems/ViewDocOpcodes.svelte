<script lang="ts">
  import { createEventDispatcher } from "svelte";

  let { isOpen = $bindable(false) }: { isOpen: boolean } = $props();

  const dispatch = createEventDispatcher();

  let searchQuery = $state("");
  let selectedOpcode = $state<string | null>(null);

  const opcodeDoc = `# Opcodes

## Mov
- **Usage:** \`Mov { dst: Reg, src: Reg }\`
- **Description:** Copy value from one register to another.

## Int
- **Usage:** \`Int { dst: Reg, ptr: RefInt }\`
- **Description:** Get an \`i32\` from the constant pool.

## Float
- **Usage:** \`Float { dst: Reg, ptr: RefFloat }\`
- **Description:** Get a \`f64\` from the constant pool.

## Bool
- **Usage:** \`Bool { dst: Reg, value: InlineBool }\`
- **Description:** Set a boolean value.

## Bytes
- **Usage:** \`Bytes { dst: Reg, ptr: RefBytes }\`
- **Description:** Get a byte array from the constant pool.

## String
- **Usage:** \`String { dst: Reg, ptr: RefString }\`
- **Description:** Get a string from the constant pool.

## Null
- **Usage:** \`Null { dst: Reg }\`
- **Description:** Nullify a register.

## Add / Sub / Mul
- **Usage:** \`Add { dst: Reg, a: Reg, b: Reg }\`  
  \`Sub { dst: Reg, a: Reg, b: Reg }\`  
  \`Mul { dst: Reg, a: Reg, b: Reg }\`
- **Description:** Arithmetic operations on two registers.

## SDiv / UDiv / SMod / UMod
- **Usage:** \`SDiv { dst: Reg, a: Reg, b: Reg }\`, etc.
- **Description:** Signed/Unsigned division and modulo.

## Shl / SShr / UShr
- **Usage:** \`Shl { dst: Reg, a: Reg, b: Reg }\`, etc.
- **Description:** Bitwise shift operations.

## And / Or / Xor
- **Usage:** \`And { dst: Reg, a: Reg, b: Reg }\`, etc.
- **Description:** Bitwise logic.

## Neg / Not
- **Usage:** \`Neg { dst: Reg, src: Reg }\`  
  \`Not { dst: Reg, src: Reg }\`
- **Description:** Negate a number or invert a boolean.

## Incr / Decr
- **Usage:** \`Incr { dst: Reg }\`  
  \`Decr { dst: Reg }\`
- **Description:** Increment or decrement a register.

## Call0 ... Call4
- **Usage:** \`Call0 { dst: Reg, fun: RefFun }\`, ..., \`Call4 { dst: Reg, fun: RefFun, args: [Reg; 4] }\`
- **Description:** Call a function with 0–4 arguments.

## CallN
- **Usage:** \`CallN { dst: Reg, fun: RefFun, args: Vec<Reg> }\`
- **Description:** Call a function with N arguments.

## CallMethod / CallThis / CallClosure
- **Usage:** \`CallMethod { dst: Reg, field: RefField, args: Vec<Reg> }\`, etc.
- **Description:** Call a method, "this" method, or closure.

## StaticClosure / InstanceClosure / VirtualClosure
- **Usage:** \`StaticClosure { dst: Reg, fun: RefFun }\`, etc.
- **Description:** Create different kinds of closures.

## GetGlobal / SetGlobal
- **Usage:** \`GetGlobal { dst: Reg, global: RefGlobal }\`  
  \`SetGlobal { global: RefGlobal, src: Reg }\`
- **Description:** Access or mutate global values.

## Field / SetField / GetThis / SetThis
- **Usage:** \`Field { dst: Reg, obj: Reg, field: RefField }\`, etc.
- **Description:** Access or set object fields.

## DynGet / DynSet
- **Usage:** \`DynGet { dst: Reg, obj: Reg, field: RefString }\`  
  \`DynSet { obj: Reg, field: RefString, src: Reg }\`
- **Description:** Dynamically access object fields.

## Conditional Jumps
- **Opcodes:** \`JTrue\`, \`JFalse\`, \`JNull\`, \`JNotNull\`, etc.
- **Usage:** \`JTrue { cond: Reg, offset: Offset }\`, etc.
- **Description:** Conditional and unconditional jumps.

## Type Conversions
- **Usage:** \`ToDyn { dst: Reg, src: Reg }\`, etc.
- **Description:** Perform type conversions.

## Label
- **Usage:** \`Label\`
- **Description:** Label for jump targets.

## Ret
- **Usage:** \`Ret { ret: Reg }\`
- **Description:** Return a value from function.

## Throw / Rethrow
- **Usage:** \`Throw { exc: Reg }\`, \`Rethrow { exc: Reg }\`
- **Description:** Throw or rethrow exceptions.

## Switch
- **Usage:** \`Switch { reg: Reg, cases: Vec<Offset>, default: Offset }\`
- **Description:** Switch-case control flow.

## NullCheck
- **Usage:** \`NullCheck { reg: Reg }\`
- **Description:** Panic if value is null.

## Trap / EndTrap
- **Usage:** \`Trap { exc: Reg, offset: Offset }\`, \`EndTrap { exc: Reg }\`
- **Description:** Try/catch-style exception handling.

## GetI8 / GetI16 / GetMem / GetArray
- **Usage:** \`GetI8 { dst: Reg, bytes: Reg, index: Reg }\`, etc.
- **Description:** Load from memory or array.

## SetI8 / SetI16 / SetMem / SetArray
- **Usage:** \`SetI8 { bytes: Reg, index: Reg, src: Reg }\`, etc.
- **Description:** Store to memory or array.

## New
- **Usage:** \`New { dst: Reg }\`
- **Description:** Allocate a new object.

## ArraySize
- **Usage:** \`ArraySize { dst: Reg, array: Reg }\`
- **Description:** Get array length.

## Type / GetType / GetTID
- **Usage:** \`Type { dst: Reg, ty: RefType }\`, etc.
- **Description:** Type introspection.

## Ref / Unref / Setref
- **Usage:** \`Ref { dst: Reg, src: Reg }\`, etc.
- **Description:** Reference counting operations.

## Enum Operations
- **Usage:** \`MakeEnum { dst: Reg, construct: RefEnumConstruct, args: Vec<Reg> }\`, etc.
- **Description:** Work with enums.

## Assert
- **Usage:** \`Assert\`
- **Description:** Trigger a debug assertion.

## RefData / RefOffset
- **Usage:** \`RefData { dst: Reg, src: Reg }\`  
  \`RefOffset { dst: Reg, reg: Reg, offset: i32 }\`
- **Description:** Data referencing operations.

## Nop
- **Usage:** \`Nop\`
- **Description:** No operation.

## Prefetch
- **Usage:** \`Prefetch { value: Reg, field: RefField, mode: i32 }\`
- **Description:** x86 prefetch hint.

## Asm
- **Usage:** \`Asm { mode: i32, value: Reg, reg: Reg }\`
- **Description:** Inline x86 assembly.`;

  interface OpcodeInfo {
    name: string;
    usage: string;
    description: string;
    opcodes?: string; // For grouped opcodes
  }

  // Parse the documentation into structured data
  function parseOpcodeDoc(): OpcodeInfo[] {
    const opcodes: OpcodeInfo[] = [];
    const sections = opcodeDoc.split('##').slice(1); // Remove the first empty element

    for (const section of sections) {
      const lines = section.trim().split('\n');
      const name = lines[0].trim();
      
      let usage = '';
      let description = '';
      let opcodesList = '';

      for (const line of lines.slice(1)) {
        if (line.includes('**Usage:**')) {
          usage = line.replace('- **Usage:**', '').trim();
        } else if (line.includes('**Description:**')) {
          description = line.replace('- **Description:**', '').trim();
        } else if (line.includes('**Opcodes:**')) {
          opcodesList = line.replace('- **Opcodes:**', '').trim();
        } else if (line.trim().startsWith('`') && usage) {
          // Multi-line usage
          usage += '\n' + line.trim();
        }
      }

      opcodes.push({
        name,
        usage,
        description,
        opcodes: opcodesList || undefined
      });
    }

    return opcodes;
  }

  const opcodes = parseOpcodeDoc();

  // Get all individual opcode names for searching
  function getAllOpcodeNames(): string[] {
    const names: string[] = [];
    
    for (const opcode of opcodes) {
      if (opcode.opcodes) {
        // Extract individual opcode names from grouped opcodes
        const individualOpcodes = opcode.opcodes.split(',').map(s => s.trim().replace(/`/g, ''));
        names.push(...individualOpcodes);
      } else if (opcode.name.includes('/')) {
        // Handle names like "Add / Sub / Mul"
        const individualOpcodes = opcode.name.split('/').map(s => s.trim());
        names.push(...individualOpcodes);
      } else {
        names.push(opcode.name);
      }
    }
    
    return names;
  }

  const allOpcodeNames = getAllOpcodeNames();

  // FIXED: Filter opcodes based on search query - removed $derived wrapper
  let filteredOpcodes = $derived.by(() => {
    if (!searchQuery) return opcodes;
    
    const query = searchQuery.toLowerCase();
    return opcodes.filter(opcode => 
      opcode.name.toLowerCase().includes(query) ||
      opcode.description.toLowerCase().includes(query) ||
      opcode.usage.toLowerCase().includes(query) ||
      (opcode.opcodes && opcode.opcodes.toLowerCase().includes(query))
    );
  });

  function closeModal() {
    isOpen = false;
    selectedOpcode = null;
  }

  function selectOpcode(opcodeName: string) {
    selectedOpcode = opcodeName;
  }

  // Handle escape key
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closeModal();
    }
  }

  // FIXED: Format usage text with null safety
  function formatUsage(usage: string | undefined): string {
    if (!usage) return '';
    return usage.replace(/`([^`]+)`/g, '<code class="bg-surface-700 px-1 rounded text-primary-300">$1</code>');
  }

  // FIXED: Get opcodes that match the selected opcode - removed $derived wrapper
  let selectedOpcodeInfo = $derived.by(() => {
    if (!selectedOpcode) return null;
    
    return opcodes.find(opcode => {
      if (opcode.name === selectedOpcode) return true;
      if (opcode.name.includes('/') && opcode.name.includes(selectedOpcode)) return true;
      if (opcode.opcodes && opcode.opcodes.includes(selectedOpcode)) return true;
      return false;
    });
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- FIXED: Changed z-index to be higher than opcode modal -->
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-[60] p-4">
    <div class="bg-white dark:bg-surface-800 rounded-lg shadow-xl max-w-6xl w-full max-h-[90vh] overflow-hidden">
      
      <!-- Header -->
      <div class="p-6 border-b border-surface-200 dark:border-surface-700 flex items-center justify-between">
        <div>
          <h2 class="text-xl font-bold text-surface-900 dark:text-surface-100">
            Opcode Documentation
          </h2>
          <p class="text-sm text-surface-600 dark:text-surface-400 mt-1">
            Reference guide for all available opcodes
          </p>
        </div>
        <button
          class="btn bg-surface-200 hover:bg-surface-300 dark:bg-surface-600 dark:hover:bg-surface-500 
            text-surface-700 dark:text-surface-200 rounded-lg px-3 py-1.5 text-sm"
          on:click={closeModal}
        >
          ✕
        </button>
      </div>

      <div class="flex h-[calc(90vh-120px)]">
        
        <!-- Sidebar - Opcode List -->
        <div class="w-80 border-r border-surface-200 dark:border-surface-700 flex flex-col">
          
          <!-- Search -->
          <div class="p-4 border-b border-surface-200 dark:border-surface-700">
            <input
              class="w-full px-3 py-2 text-sm border border-surface-300 dark:border-surface-600 rounded-lg 
                bg-white dark:bg-surface-700 text-surface-900 dark:text-surface-100
                focus:border-primary-500 focus:ring-1 focus:ring-primary-500"
              type="text"
              placeholder="Search opcodes..."
              bind:value={searchQuery}
            />
          </div>

          <!-- Opcode List -->
          <div class="flex-1 overflow-y-auto">
            {#each filteredOpcodes as opcode}
              <button
                class="w-full text-left px-4 py-3 hover:bg-surface-100 dark:hover:bg-surface-700 
                  border-b border-surface-100 dark:border-surface-700
                  {selectedOpcode === opcode.name ? 'bg-primary-100 dark:bg-primary-900' : ''}"
                on:click={() => selectOpcode(opcode.name)}
              >
                <div class="font-mono text-sm font-medium text-surface-900 dark:text-surface-100">
                  {opcode.name}
                </div>
                <div class="text-xs text-surface-600 dark:text-surface-400 mt-1 truncate">
                  {opcode.description}
                </div>
                {#if opcode.opcodes}
                  <div class="text-xs text-primary-600 dark:text-primary-400 mt-1">
                    {opcode.opcodes}
                  </div>
                {/if}
              </button>
            {/each}
            
            {#if filteredOpcodes.length === 0}
              <div class="p-4 text-center text-surface-500 dark:text-surface-400">
                <p>No opcodes found matching "{searchQuery}"</p>
              </div>
            {/if}
          </div>
        </div>

        <!-- Main Content -->
        <div class="flex-1 overflow-y-auto">
          {#if selectedOpcodeInfo}
            <div class="p-6">
              <h3 class="text-2xl font-bold text-surface-900 dark:text-surface-100 mb-4 font-mono">
                {selectedOpcodeInfo.name}
              </h3>
              
              {#if selectedOpcodeInfo.opcodes}
                <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
                  <h4 class="text-sm font-semibold text-blue-800 dark:text-blue-200 mb-2">Related Opcodes:</h4>
                  <p class="text-sm text-blue-700 dark:text-blue-300 font-mono">{selectedOpcodeInfo.opcodes}</p>
                </div>
              {/if}

              <div class="space-y-6">
                <div>
                  <h4 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-3">Usage</h4>
                  <div class="bg-surface-50 dark:bg-surface-900 rounded-lg p-4 border border-surface-200 dark:border-surface-700">
                    <pre class="text-sm text-surface-900 dark:text-surface-100 font-mono whitespace-pre-wrap overflow-x-auto">{@html formatUsage(selectedOpcodeInfo.usage)}</pre>
                  </div>
                </div>

                <div>
                  <h4 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-3">Description</h4>
                  <div class="prose prose-sm dark:prose-invert max-w-none">
                    <p class="text-surface-700 dark:text-surface-300">{selectedOpcodeInfo.description}</p>
                  </div>
                </div>

                <!-- Parameter Types Guide -->
                <div class="mt-8">
                  <h4 class="text-lg font-semibold text-surface-900 dark:text-surface-100 mb-3">Parameter Types Reference</h4>
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-sm">
                    <div class="space-y-2">
                      <div><code class="bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded">Reg</code> - Register index</div>
                      <div><code class="bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded">RefInt</code> - Integer constant reference</div>
                      <div><code class="bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded">RefFloat</code> - Float constant reference</div>
                      <div><code class="bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded">RefString</code> - String constant reference</div>
                    </div>
                    <div class="space-y-2">
                      <div><code class="bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded">RefFun</code> - Function reference</div>
                      <div><code class="bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded">RefField</code> - Field reference</div>
                      <div><code class="bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded">RefType</code> - Type reference</div>
                      <div><code class="bg-surface-200 dark:bg-surface-700 px-2 py-1 rounded">Offset</code> - Jump offset</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          {:else}
            <div class="p-6 h-full flex items-center justify-center">
              <div class="text-center text-surface-500 dark:text-surface-400">
                <svg class="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" 
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <h3 class="text-lg font-medium mb-2">Select an Opcode</h3>
                <p class="text-sm">Choose an opcode from the list to view its documentation</p>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}