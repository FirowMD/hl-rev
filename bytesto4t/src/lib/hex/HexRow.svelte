<script lang="ts">
  import HexCell from './HexCell.svelte';
  
  export let offset: number;
  export let bytes: Uint8Array;
  export let bytesPerRow: number = 16;
  export let selectedIndex: number | null = null;
  
  $: rowBytes = bytes.slice(0, bytesPerRow);
  $: offsetHex = offset.toString(16).padStart(8, '0').toUpperCase();
  
  // ASCII representation
  $: ascii = Array.from(rowBytes)
    .map(byte => byte >= 32 && byte <= 126 ? String.fromCharCode(byte) : '.');
    
  function handleCellSelect(index: number) {
    selectedIndex = index;
  }
</script>

<div class="flex gap-2" role="row">
  <!-- Offset -->
  <div class="text-surface-400 w-24" role="rowheader">
    {offsetHex}
  </div>
  
  <!-- Hex values -->
  <div class="flex gap-1" role="group">
    {#each rowBytes as byte, i}
      <HexCell
        value={byte}
        isSelected={selectedIndex === i}
        on:select={() => handleCellSelect(i)}
        on:change
      />
    {/each}
  </div>
  
  <!-- ASCII representation -->
  <div class="pl-4 font-mono" role="group">
    {ascii.join('')}
  </div>
</div>
