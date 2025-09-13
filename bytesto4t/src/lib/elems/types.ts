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
import { writable } from 'svelte/store';


export const functionsRefreshKey = writable(0);
export const mainPanelTab = writable<string>("dashboard");
export const functionToEdit = writable<number | null>(null);
export const typeToEdit = writable<number | null>(null);
export const globalToEdit = writable<number | null>(null);
export const nativeToEdit = writable<number | null>(null);
export const constantToEdit = writable<number | null>(null);
export const stringToEdit = writable<number | null>(null);
export const intToEdit = writable<number | null>(null);
export const floatToEdit = writable<number | null>(null);
export const byteToEdit = writable<number | null>(null);

export const typesRefreshKey = writable(0);
export const globalsRefreshKey = writable(0);
export const nativesRefreshKey = writable(0);
export const constantsRefreshKey = writable(0);
export const stringsRefreshKey = writable(0);
export const intsRefreshKey = writable(0);
export const floatsRefreshKey = writable(0);
export const bytesRefreshKey = writable(0);

export interface NewTypeData {
  type_kind: string;
  name?: string;
  super_type?: number;
  global?: number;
  inner_type?: number;
  args?: number[];
  ret?: number;
  fields?: Array<{name: string, field_type: number}>;
  protos?: Array<{name: string, findex: number, pindex: number}>;
  constructs?: Array<{name: string, params: number[]}>;
}

export interface NewGlobalData {
  global_type: number;
}

export interface NewNativeData {
  lib: string;
  name: string;
  signature_type: number;
  findex?: number;
}

export interface NewConstantData {
  global: number;
  fields: number[];
}

export interface FunctionSaveEvent {
  name: string;
  index?: number;
  type: number;
  registers: number[];
  opcodes: any[];
}

export interface FunctionEditEvent extends FunctionSaveEvent {
  functionIndex: number;
}

export interface TypeSaveEvent extends NewTypeData {}

export interface TypeEditEvent extends NewTypeData {
  typeIndex: number;
}

export interface GlobalSaveEvent extends NewGlobalData {}

export interface GlobalEditEvent extends NewGlobalData {
  globalIndex: number;
}

export interface NativeSaveEvent extends NewNativeData {}

export interface NativeEditEvent extends NewNativeData {
  nativeIndex: number;
}

export interface ConstantSaveEvent extends NewConstantData {}

export interface ConstantEditEvent extends NewConstantData {
  constantIndex: number;
}

export function triggerEditFunction(index: number) {
  functionToEdit.set(index);
}

export function triggerEditType(index: number) {
  typeToEdit.set(index);
}

export function triggerEditGlobal(index: number) {
  globalToEdit.set(index);
}

export function triggerEditNative(index: number) {
  nativeToEdit.set(index);
}

export function triggerEditConstant(index: number) {
  constantToEdit.set(index);
}

export interface ConstructorComponent {
  editFunction(index: number): void;
  editType(index: number): void;
  editGlobal(index: number): void;
  editNative(index: number): void;
  editConstant(index: number): void;
}

export enum ElementType {
  FUNCTION = 'function',
  TYPE = 'type',
  GLOBAL = 'global',
  NATIVE = 'native',
  CONSTANT = 'constant'
}

export interface ElementReference {
  type: ElementType;
  index: number;
  name?: string;
  description?: string;
}

export type ConstructorTab = 'functions' | 'types' | 'globals' | 'natives' | 'constants';

export type ModalMode = 'create' | 'edit';

export interface ValidationState {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

export interface FormFieldState<T = string> {
  value: T;
  error?: string;
  touched: boolean;
  required: boolean;
}

export interface FormState<T = Record<string, any>> {
  data: T;
  validation: ValidationState;
  isDirty: boolean;
  isSubmitting: boolean;
}

export interface ReferenceOption {
  idx: number;
  value: string;
  label?: string;
  description?: string;
}

export interface TypeOption extends ReferenceOption {
  name: string;
  kind?: string; // e.g., "fun", "obj", "primitive"
}

export interface StringOption extends ReferenceOption {}

export interface GlobalOption extends ReferenceOption {}

export interface FunctionOption extends ReferenceOption {}

export interface PopoverState {
  activePopover: string | null;
  searchQueries: Record<string, string>;
}

export interface ConstructorProps {
  modalMode: ModalMode;
  editIndex: number | null;
}

export type ElementSaveHandler<T = any> = (event: CustomEvent<T>) => Promise<void>;
export type ElementEditHandler<T = any> = (event: CustomEvent<T>) => Promise<void>;

export interface ConstructorEvents {
  functionSave: FunctionSaveEvent;
  functionEdit: FunctionEditEvent;
  typeSave: TypeSaveEvent;
  typeEdit: TypeEditEvent;
  globalSave: GlobalSaveEvent;
  globalEdit: GlobalEditEvent;
  nativeSave: NativeSaveEvent;
  nativeEdit: NativeEditEvent;
  constantSave: ConstantSaveEvent;
  constantEdit: ConstantEditEvent;
}

export interface ConstructorTabConfig {
  id: ConstructorTab;
  label: string;
  icon: string;
  description: string;
  component: any;
}

export interface CommandResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}

export interface ConstructorError {
  type: 'validation' | 'backend' | 'network';
  message: string;
  field?: string;
  code?: string;
}

export interface ConstructorSuccess {
  type: 'create' | 'update' | 'delete';
  elementType: ElementType;
  message: string;
}

export interface ConstructorState {
  currentTab: ConstructorTab;
  editMode: Record<ConstructorTab, ModalMode>;
  editIndex: Record<ConstructorTab, number | null>;
  loading: Record<ConstructorTab, boolean>;
  errors: Record<ConstructorTab, ConstructorError[]>;
}

export interface DataLoadingState {
  types: boolean;
  strings: boolean;
  functions: boolean;
  globals: boolean;
  natives: boolean;
  constants: boolean;
}

export interface ConstructorContext {
  refreshData: () => Promise<void>;
  showNotification: (message: string, type: 'success' | 'error' | 'warning') => void;
  setLoading: (loading: boolean) => void;
  getCurrentTab: () => ConstructorTab;
  switchTab: (tab: ConstructorTab) => void;
}