export type FunctionListEntry = {
  name: string;
  findex: number;
  index: number;
};

export function parseFunctionListEntry(value: string): FunctionListEntry {
  const indexSeparator = value.lastIndexOf("@");
  const findexSeparator = value.lastIndexOf("@", indexSeparator - 1);
  if (findexSeparator < 0 || indexSeparator <= findexSeparator + 1) {
    throw new Error(`Invalid function list entry: ${value}`);
  }

  const findex = Number(value.slice(findexSeparator + 1, indexSeparator));
  const index = Number(value.slice(indexSeparator + 1));
  if (!Number.isSafeInteger(findex) || findex < 0 || !Number.isSafeInteger(index) || index < 0) {
    throw new Error(`Invalid function list indexes: ${value}`);
  }

  return { name: value.slice(0, findexSeparator), findex, index };
}
