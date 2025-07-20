import { atom } from 'nanostores';
import type { VariableValue } from '../types';

export const globalVariablesStore = atom<Record<string, VariableValue>>({});

export const updateGlobalVariables = (variables: Record<string, VariableValue>) => {
  globalVariablesStore.set(variables);
};

export const formatVariableValue = (value: VariableValue): string => {
  // Check for object types first
  if (typeof value === 'object' && value !== null) {
    if ('Integer' in value) return value.Integer.toString();
    if ('Float' in value) return value.Float.toFixed(2);
    if ('Bool' in value) return value.Bool ? 'true' : 'false';
    if ('Str' in value) return value.Str;
  }
  
  // Check for Decimal as array [sign, numerator, denominator]
  if (Array.isArray(value) && value.length === 3) {
    const [sign, num, den] = value;
    const decimal = (sign * num) / den;
    return decimal.toFixed(2);
  }
  
  return 'nil';
};