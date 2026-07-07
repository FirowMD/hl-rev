--[[
    Supports only 64-bit architecture
    Current script is for following hashlink's implementation:

    typedef struct {
        pchar *file;
        hl_code *code;
        hl_module *m;
        vdynamic *ret;
        int file_time;
    } main_context;

    typedef struct {
        hl_code *code;
        int codesize;
        int globals_size;
        int *globals_indexes;
        unsigned char *globals_data;
        void **functions_ptrs;
        int *functions_indexes;
        void *jit_code;
        hl_code_hash *hash;
        hl_debug_infos *jit_debug;
        jit_ctx *jit_ctx;
        hl_module_context ctx;
    } hl_module;
]]--

local hashlinkVersion = -1

-- Helper functions from export_functions.lua
function convertAddressToScanData(address)
    local addr = tonumber(address, 16)
    if not addr then
        return nil
    end

    local bytes = qwordToByteTable(addr)
    
    local pattern = {}
    for i=1, #bytes do
        pattern[i] = string.format("%02X", bytes[i])
    end
    
    return table.concat(pattern, " ")
end

local function isUtf16Match(bytes, startIndex, searchString)
    for i = 1, #searchString do
        local char = string.byte(searchString:sub(i,i))
        local byteIndex = startIndex + (i-1)*2
        
        if bytes[byteIndex] ~= char or bytes[byteIndex + 1] ~= 0 then
            return false
        end
    end
    return true
end

local function searchUtf16StringInRegion(bytes, searchString, baseAddress)
    local searchLength = #searchString * 2
    
    for i = 1, #bytes - searchLength do
        if isUtf16Match(bytes, i, searchString) then
            return string.format("%X", baseAddress + i - 1)
        end
    end
    
    return nil
end

function findHlbootdatAddress()
    local EXPECTED_STRING = "hlboot.dat"
    local IMAGE_TYPE = 0x1000000

    local regions = enumMemoryRegions()
    
    for _, region in ipairs(regions) do
        if region.Type == IMAGE_TYPE then
            local bytes = readBytes(region.BaseAddress, region.RegionSize, true)
            
            if bytes then
                local address = searchUtf16StringInRegion(bytes, EXPECTED_STRING, region.BaseAddress)
                if address then
                    return address
                end
            end
        end
    end

    return nil, "Could not find 'hlboot.dat' string in memory!"
end

function setup_hashlink_version(structure_address)
    local struct_addr = tonumber(structure_address, 16)
    -- addr: hl_code *code;
    local addr = readQword(struct_addr + 0x8)
    local possible_values = {3, 4, 5}
    for _, value in ipairs(possible_values) do
        current_value = readInteger(addr)
        if current_value == value then
            hashlinkVersion = value
            return true
        end
    end

    return false
end

function getHashlinkNfunctions(structure_address)
    local NFUNCTIONS_OFFSET = 28
    if hashlinkVersion >= 4 then
        NFUNCTIONS_OFFSET = 32
    end

    local struct_addr = tonumber(structure_address, 16)
    -- addr: hl_code *code;
    local addr = readQword(struct_addr + 0x8)
    local nfunctions = readInteger(addr + NFUNCTIONS_OFFSET)

    return nfunctions
end

function getStructureAddress(hlboot_dat_address)
    local scandata = convertAddressToScanData(hlboot_dat_address)
    if not scandata then
        return nil
    end

    local results = AOBScan(scandata, "+W")
    if not results or results.Count == 0 then
        if results then results.destroy() end
        return nil
    end

    local structureAddress = nil
    for i = 0, results.Count - 1 do
        local addr = results[i]
        if setup_hashlink_version(addr) then
            structureAddress = addr
            break
        end
    end

    results.destroy()
    return structureAddress
end

function getListOfFunctions(structure_address, nfunctions)
    local result = {}
    local struct_addr = tonumber(structure_address, 16)
    local hl_module_pointer = readQword(struct_addr + 0x10)
    local functions_pointer = readQword(hl_module_pointer + 0x20)
    
    local bytes = readBytes(functions_pointer, nfunctions * 8, true)
    if not bytes then
        return result
    end
    
    for i = 1, #bytes, 8 do
        local function_address = byteTableToQword({
            bytes[i],
            bytes[i + 1],
            bytes[i + 2],
            bytes[i + 3],
            bytes[i + 4],
            bytes[i + 5],
            bytes[i + 6],
            bytes[i + 7] 
        })

        table.insert(result, string.format("%X", function_address))
    end
    
    return result
end

-- Window implementation
local ExportFunctionsForm = nil

function createExportFunctionsWindow()
    -- Create main form
    ExportFunctionsForm = createForm(false)
    ExportFunctionsForm.Caption = "Export Hashlink Functions"
    ExportFunctionsForm.BorderStyle = bsSingle
    ExportFunctionsForm.Width = 400
    ExportFunctionsForm.Height = 300
    ExportFunctionsForm.Position = poScreenCenter

    -- Create output path edit
    local lblPath = createLabel(ExportFunctionsForm)
    lblPath.Caption = "Output file path:"
    lblPath.Left = 10
    lblPath.Top = 10

    local edtPath = createEdit(ExportFunctionsForm)
    edtPath.Left = 10
    edtPath.Top = 30
    edtPath.Width = 300
    edtPath.Text = "D:/functions.txt"

    -- Create Browse button
    local btnBrowse = createButton(ExportFunctionsForm)
    btnBrowse.Caption = "Browse"
    btnBrowse.Left = 320
    btnBrowse.Top = 28
    btnBrowse.Width = 60
    btnBrowse.OnClick = function()
        local saveDialog = createSaveDialog(ExportFunctionsForm)
        saveDialog.Title = "Save functions list"
        saveDialog.Filter = "Text files (*.txt)|*.txt|All files (*.*)|*.*"
        saveDialog.DefaultExt = "txt"
        saveDialog.FileName = edtPath.Text
        
        if saveDialog.Execute() then
            edtPath.Text = saveDialog.FileName
        end
        saveDialog.destroy()
    end

    -- Create memo for log
    local memoLog = createMemo(ExportFunctionsForm)
    memoLog.Left = 10
    memoLog.Top = 60
    memoLog.Width = 380
    memoLog.Height = 180
    memoLog.ScrollBars = ssVertical
    memoLog.ReadOnly = true

    -- Create export button
    local btnExport = createButton(ExportFunctionsForm)
    btnExport.Caption = "Export Functions"
    btnExport.Left = 10
    btnExport.Top = 250
    btnExport.Width = 100
    btnExport.OnClick = function()
        memoLog.Clear()
        
        local function log(text)
            memoLog.Lines.Add(text)
        end

        -- Export process
        local hlboot_address, error = findHlbootdatAddress()
        if not hlboot_address then
            log("Error: " .. error)
            return
        end
        log("HLBoot address: " .. hlboot_address)

        local structure_address = getStructureAddress(hlboot_address)
        if not structure_address then
            log("Error: Could not find pointer to 'hlboot.dat' address!")
            return
        end
        log("Structure address: " .. structure_address)
        log("Hashlink version: " .. hashlinkVersion)

        local nfunctions = getHashlinkNfunctions(structure_address)
        log("Function number: " .. nfunctions)

        local function_list = getListOfFunctions(structure_address, nfunctions)
        log("Functions successfully extracted: " .. #function_list)

        -- Save to file
        local file = io.open(edtPath.Text, "w")
        if not file then
            log("Error: Could not open output file!")
            return
        end

        for _, function_address in ipairs(function_list) do
            file:write(function_address .. "\n")
        end
        file:close()

        log("Saved to " .. edtPath.Text)
    end

    ExportFunctionsForm.show()
end

function findComponentByName(parent,name)
	local count = parent.ComponentCount
	for i = 0,count do if parent.Component[i].Caption == name then return parent.Component[i] end end
	return nil
end

-- Add menu item to Tools
function addExportFunctionsMenuItem()
	local parent = findComponentByName(getMemoryViewForm(),'Tools')
    -- Find Tools menu
    for i=0, MainForm.Menu.Items.Count-1 do
        if MainForm.Menu.Items[i].Caption == "Tools" then
            parent = MainForm.Menu.Items[i]
            break
        end
    end

    if parent == nil then return end

    -- Create menu item
    local exportFunctionsMenuItem = createMenuItem(parent)
    exportFunctionsMenuItem.Caption = "Export Hashlink Functions"
    exportFunctionsMenuItem.OnClick = createExportFunctionsWindow
    exportFunctionsMenuItem.ImageIndex = 47  -- Using save icon (similar to export functionality)
    parent.add(exportFunctionsMenuItem)
end

-- Initialize
addExportFunctionsMenuItem()
