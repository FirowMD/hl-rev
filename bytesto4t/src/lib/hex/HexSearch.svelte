<script lang="ts">
  const { data, onSearch } = $props<{
    data: Uint8Array;
    onSearch: (results: number[]) => void;
  }>();

  let searchValue = $state('');

  async function handleSearch() {
    if (!searchValue) {
      onSearch([]);
      return;
    }
    
    const searchHex = searchValue.replace(/\s+/g, '');
    const isHexSearch = /^[0-9A-Fa-f\s]+$/.test(searchValue);
    
    const searchBytes = isHexSearch
      ? new Uint8Array(searchHex.match(/.{1,2}/g)?.map(byte => parseInt(byte, 16)) || [])
      : new TextEncoder().encode(searchValue);
    
    const results: number[] = [];
    for (let i = 0; i < data.length - searchBytes.length + 1; i++) {
      let found = true;
      for (let j = 0; j < searchBytes.length; j++) {
        if (data[i + j] !== searchBytes[j]) {
          found = false;
          break;
        }
      }
      if (found) results.push(i);
    }
    
    onSearch(results);
  }
</script>

<div class="flex gap-2 p-2">
  <input
    type="text"
    bind:value={searchValue}
    placeholder="Search (hex or text)"
    class="input"
    on:keydown={(e) => e.key === 'Enter' && handleSearch()}
  />
  <button class="btn variant-filled-primary" on:click={handleSearch}>
    Search
  </button>
</div> 