# Cheat Engine Scripts

## export_functions.lua

This script saves a list of function addresses that are translated from HashLink bytecode to native machine code.

To use the script, you need to specify the output file path by changing the `OUTPUT_FILE_PATH` variable:

```lua
local OUTPUT_FILE_PATH = "D:/functions.txt"
```