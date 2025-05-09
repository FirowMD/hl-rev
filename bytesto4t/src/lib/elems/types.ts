export interface FileData {
  buffer: Uint8Array;
  size: number;
  name: string;
}

export interface HistoryItem {
  name: string;
  typ: string;
  timestamp: string;
}

export interface AIDecompilation {
  function_name: string;
  result: string;
  timestamp: string;
  model: string;
}

export interface BytecodeItemSelectedEvent {
  name: string;
  type: string;
}

export interface AIDecompilationReplacedEvent {
  functionName: string;
  result: string;
}

export interface FileStatus {
  name: string;
  lines: number;
}

export interface Reference {
  elementIndex: number;
  references: string[];
}

declare global {
  interface WindowEventMap {
    'bytecode-item-selected': CustomEvent<BytecodeItemSelectedEvent>;
    'ai-decompilation-replaced': CustomEvent<AIDecompilationReplacedEvent>;
    'ai-decompilation-removed': CustomEvent<{functionName: string}>;
  }
}