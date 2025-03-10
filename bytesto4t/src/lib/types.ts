// File and binary data types
export interface FileData {
  buffer: Uint8Array;
  size: number;
  name: string;
}

// History related types
export interface HistoryItem {
  name: string;
  typ: string;
  timestamp: string;
}

// AI Decompilation types
export interface AIDecompilation {
  function_name: string;
  result: string;
  timestamp: string;
  model: string;
}

// Event types
export interface BytecodeItemSelectedEvent {
  name: string;
  type: string;
}

export interface AIDecompilationReplacedEvent {
  functionName: string;
  result: string;
}

// Status types
export interface FileStatus {
  name: string;
  lines: number;
}

// Reference types
export interface Reference {
  elementIndex: number;
  references: string[];
}

// Custom Events
declare global {
  interface WindowEventMap {
    'bytecode-item-selected': CustomEvent<BytecodeItemSelectedEvent>;
    'ai-decompilation-replaced': CustomEvent<AIDecompilationReplacedEvent>;
  }
} 