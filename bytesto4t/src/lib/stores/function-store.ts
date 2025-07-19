import { writable } from 'svelte/store';
import type { FunctionEdit } from '../types/function-types';

export const functionStore = writable({
  name: "",
  index: "",
  typeIdx: "",
  registers: [0],
  opcodes: [] as any[]
});

export const resetFunction = () => {
  functionStore.set({
    name: "",
    index: "",
    typeIdx: "",
    registers: [0],
    opcodes: []
  });
};

export const loadFunction = (fun: FunctionEdit) => {
  functionStore.update(state => ({
    ...state,
    name: String(fun.name),
    typeIdx: String(fun.t),
    index: String(fun.findex),
    registers: fun.regs.map(t => t),
    opcodes: fun.ops
  }));
};
