<script lang="ts">
  const { bytesPerRow } = $props<{ bytesPerRow: number }>();
</script>

<div class="flex gap-2 font-mono text-surface-400 sticky top-0 bg-surface-900/90 backdrop-blur-sm p-2 z-10">
  <div class="w-24">Offset</div>
  <div class="flex gap-1">
    {#each Array(bytesPerRow) as _, i}
      <div class="w-[26px] text-center">
        {i.toString(16).padStart(2, '0').toUpperCase()}
      </div>
    {/each}
  </div>
  <div class="pl-4">ASCII</div>
</div> 