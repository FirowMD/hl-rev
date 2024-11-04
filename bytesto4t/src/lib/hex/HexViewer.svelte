<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SelectEvent } from './types';
  import HexCell from './HexCell.svelte';
  
  const dispatch = createEventDispatcher<{ 
    select: SelectEvent;
    selectionStart: { row: number; col: number };
    selectionMove: { row: number; col: number };
    selectionEnd: void;
    scroll: number;
  }>();

  const { 
    data, 
    bytesPerRow, 
    searchResults = [],
    selection,
    scrollPosition = 0
  } = $props<{
    data: Uint8Array;
    bytesPerRow: number;
    searchResults?: number[];
    selection: {
      start: { row: number; col: number } | null;
      end: { row: number; col: number } | null;
      isSelecting: boolean;
    };
    scrollPosition?: number;
  }>();

  // Virtual scrolling state
  let containerElement = $state<HTMLDivElement | null>(null);
  let virtualScroll = $state({
    start: 0,
    viewportHeight: 0,
  });

  const ROW_HEIGHT = 24;
  const CHUNK_SIZE = 100;
  const BUFFER_SIZE = 5;

  let totalRows = $derived(Math.ceil(data.length / bytesPerRow));
  
  let visibleRange = $derived({
    start: !containerElement ? 0 : Math.max(0, virtualScroll.start - BUFFER_SIZE),
    end: !containerElement ? CHUNK_SIZE : Math.min(
      totalRows,
      virtualScroll.start + Math.min(
        Math.ceil(virtualScroll.viewportHeight / ROW_HEIGHT) + BUFFER_SIZE,
        CHUNK_SIZE
      )
    )
  });

  // Handle scroll events with throttling
  let scrollTimeout: number | null = null;
  function handleScroll() {
    if (!containerElement || scrollTimeout) return;
    
    scrollTimeout = window.setTimeout(() => {
      if (!containerElement) return;
      
      virtualScroll.start = Math.floor(containerElement.scrollTop / ROW_HEIGHT);
      virtualScroll.viewportHeight = containerElement.clientHeight;
      scrollTimeout = null;

      dispatch('scroll', containerElement.scrollTop);
    }, 16);
  }

  // Restore scroll position on mount or when it changes
  $effect(() => {
    if (containerElement && scrollPosition) {
      containerElement.scrollTop = scrollPosition;
    }
  });

  function handleMouseDown(row: number, col: number) {
    dispatch('selectionStart', { row, col });
  }

  function handleMouseEnter(row: number, col: number) {
    dispatch('selectionMove', { row, col });
  }

  $effect(() => {
    if (!containerElement) return;
    virtualScroll.viewportHeight = containerElement.clientHeight;
    handleScroll();
  });

  function isInSelection(row: number, col: number): boolean {
    if (!selection.start || !selection.end) return false;

    const start = {
      row: Math.min(selection.start.row, selection.end.row),
      col: Math.min(selection.start.col, selection.end.col)
    };
    const end = {
      row: Math.max(selection.start.row, selection.end.row),
      col: Math.max(selection.start.col, selection.end.col)
    };

    const pos = row * bytesPerRow + col;
    const startPos = start.row * bytesPerRow + start.col;
    const endPos = end.row * bytesPerRow + end.col;

    return pos >= startPos && pos <= endPos;
  }
</script>

<div 
  bind:this={containerElement}
  class="flex-1 overflow-y-auto relative font-mono"
  on:scroll={handleScroll}
  style="height: calc(100vh - 12rem);"
>
  <div class="absolute w-full" style="height: {totalRows * ROW_HEIGHT}px">
    <div class="absolute w-full" style="top: {visibleRange.start * ROW_HEIGHT}px">
      {#each Array(visibleRange.end - visibleRange.start) as _, index}
        {@const rowIndex = visibleRange.start + index}
        {@const rowOffset = rowIndex * bytesPerRow}
        {@const rowData = data.slice(rowOffset, rowOffset + bytesPerRow)}
        
        <div class="flex gap-2 h-[24px] hover:bg-surface-500/10" role="row">
          <!-- Offset -->
          <div class="text-surface-400 w-24 pl-2" role="rowheader">
            {rowOffset.toString(16).padStart(8, '0').toUpperCase()}
          </div>
          
          <!-- Hex values -->
          <div class="flex gap-1" role="group">
            {#each rowData as byte, colIndex}
              <HexCell
                value={byte}
                isHighlighted={searchResults.includes(rowOffset + colIndex)}
                isInSelection={isInSelection(rowIndex, colIndex)}
                on:mousedown={() => handleMouseDown(rowIndex, colIndex)}
                on:mouseenter={() => handleMouseEnter(rowIndex, colIndex)}
                on:select={() => dispatch('select', { 
                  offset: rowOffset + colIndex,
                  value: byte 
                })}
              />
            {/each}
            
            <!-- Padding for incomplete rows -->
            {#if rowData.length < bytesPerRow}
              {#each Array(bytesPerRow - rowData.length) as _}
                <div class="w-[26px] text-center">··</div>
              {/each}
            {/if}
          </div>
          
          <!-- ASCII representation -->
          <div class="pl-4 font-mono" role="group">
            {#each rowData as byte}
              <span class="inline-block">
                {byte >= 32 && byte <= 126 ? String.fromCharCode(byte) : '.'}
              </span>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  </div>
</div> 