export interface FunctionEdit {
  name: number;
  t: number;
  findex: number;
  ops: any[];
  regs: number[];
}

export interface TypeInfo {
  idx: number;
  name: string;
}

export interface StringInfo {
  idx: number;
  value: string;
}

export interface EditingOpcode {
  idx: number;
  key: string;
  params: Record<string, string>;
  open: boolean;
  query: string;
}

export interface RegisterPopover {
  open: boolean;
  query: string;
  ref: HTMLDivElement | null;
  filteredTypes: TypeInfo[];
}
