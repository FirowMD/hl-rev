<script lang="ts">
  import HexHeader from './hex/HexHeader.svelte';
  import HexSearch from './hex/HexSearch.svelte';
  import HexViewer from './hex/HexViewer.svelte';
  import type { SelectEvent } from './hex/types';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    scroll: { position: number };
  }>();

  const { 
    data, 
    bytesPerRow = 16,
    scrollPosition = 0,
    targetOffset = -1 
  } = $props<{
    data: Uint8Array;
    bytesPerRow?: number;
    scrollPosition?: number;
    targetOffset?: number;
  }>();

  let searchResults = $state<number[]>([]);
  let selection = $state({
    start: null as { row: number; col: number } | null,
    end: null as { row: number; col: number } | null,
    isSelecting: false
  });

  function handleSearch(results: number[]) {
    searchResults = results;
  }

  function handleScroll(event: CustomEvent<number>) {
    dispatch('scroll', { position: event.detail });
  }

  function handleSelect(event: CustomEvent<SelectEvent>) {
    console.log('Selected:', event.detail);
  }

  function handleSelectionStart(event: CustomEvent<{ row: number; col: number }>) {
    selection = {
      ...selection,
      isSelecting: true,
      start: event.detail,
      end: event.detail
    };
  }

  function handleSelectionMove(event: CustomEvent<{ row: number; col: number }>) {
    if (selection.isSelecting) {
      selection = {
        ...selection,
        end: event.detail
      };
    }
  }

  function handleMouseUp() {
    selection = {
      ...selection,
      isSelecting: false
    };
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.ctrlKey || event.metaKey) {
      if (event.key === 'c') {
        event.preventDefault();
        copySelection();
      }
      if (event.key === 'a') {
        event.preventDefault();
        selection.start = { row: 0, col: 0 };
        selection.end = { 
          row: Math.floor(data.length / bytesPerRow),
          col: (data.length % bytesPerRow) - 1
        };
      }
    }
  }

  function copySelection() {
    if (!selection.start || !selection.end) return;

    const start = {
      row: Math.min(selection.start.row, selection.end.row),
      col: Math.min(selection.start.col, selection.end.col)
    };
    const end = {
      row: Math.max(selection.start.row, selection.end.row),
      col: Math.max(selection.start.col, selection.end.col)
    };

    const startPos = start.row * bytesPerRow + start.col;
    const endPos = end.row * bytesPerRow + end.col;
    
    const selectedBytes = data.slice(startPos, endPos + 1);
    const hexString = [...selectedBytes]
      .map(byte => byte.toString(16).padStart(2, '0'))
      .join(' ');
    
    navigator.clipboard.writeText(hexString.toUpperCase());
  }

  $effect(() => {
    if (targetOffset >= 0) {
      const targetRow = Math.floor(targetOffset / bytesPerRow);
      const scrollTop = targetRow * 24;
      dispatch('scroll', { position: scrollTop });
    }
  });
</script>

<svelte:window 
  on:mouseup={handleMouseUp}
  on:keydown={handleKeyDown}
/>

<div class="flex flex-col gap-4 h-full">
  <HexSearch {data} onSearch={handleSearch} />
  <HexHeader {bytesPerRow} />
  <HexViewer 
    {data} 
    {bytesPerRow} 
    {searchResults}
    {selection}
    {scrollPosition}
    targetOffset={targetOffset}
    on:select={handleSelect}
    on:selectionStart={handleSelectionStart}
    on:selectionMove={handleSelectionMove}
    on:selectionEnd={handleMouseUp}
    on:scroll={handleScroll}
  />
</div> 