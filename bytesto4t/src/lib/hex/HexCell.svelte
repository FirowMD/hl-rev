<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher<{
    select: number;
    mousedown: void;
    mouseenter: void;
  }>();

  const { 
    value,
    isSelected = false,
    isHighlighted = false,
    isInSelection = false,
    isTargetOffset = false 
  } = $props<{
    value: number;
    isSelected?: boolean;
    isHighlighted?: boolean;
    isInSelection?: boolean;
    isTargetOffset?: boolean;
  }>();

  let hex = $derived(value.toString(16).padStart(2, '0').toUpperCase());
</script>

<button
  type="button"
  role="gridcell"
  class="inline-block w-[26px] text-center select-none cursor-pointer
    {isSelected ? 'bg-primary-500/20' : ''}
    {isHighlighted ? 'bg-secondary-500/20' : ''}
    {isInSelection ? 'bg-primary-500/10' : ''}
    {isTargetOffset ? 'bg-tertiary-500/50' : ''}
    hover:bg-primary-500/10"
  on:click={() => dispatch('select', value)}
  on:mousedown={() => dispatch('mousedown')}
  on:mouseenter={() => dispatch('mouseenter')}
>
  {hex}
</button> 