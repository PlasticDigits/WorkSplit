---
context_files: []
output_dir: src/
output_file: calculator.ts
---

# Create Calculator Logic Module

## Overview
Create a TypeScript module with calculator logic that will be used by the React component.

## Requirements
- Pure functions for arithmetic operations
- Support for building expressions as strings
- Proper TypeScript types
- Handle division by zero gracefully

## Functions to Implement

### `add(a: number, b: number): number`
Returns the sum of two numbers.

### `subtract(a: number, b: number): number`
Returns the difference of two numbers.

### `multiply(a: number, b: number): number`
Returns the product of two numbers.

### `divide(a: number, b: number): number`
Returns the quotient of two numbers. Throws Error("Division by zero") if b is 0.

### `calculate(expression: string): number`
Parses and evaluates a simple arithmetic expression.
- Supports +, -, *, / operators
- Supports decimal numbers
- Supports negative numbers
- Returns NaN for invalid expressions
- Use a simple approach: split by operators and evaluate left to right

### `formatNumber(num: number): string`
Formats a number for display:
- Limits decimal places to 10
- Removes trailing zeros
- Handles very large/small numbers with exponential notation

## Types to Export

```typescript
export type Operator = '+' | '-' | '*' | '/';

export interface CalculatorState {
  display: string;
  previousValue: number | null;
  operator: Operator | null;
  waitingForOperand: boolean;
}
```

## Example Usage

```typescript
import { add, subtract, calculate, formatNumber } from './calculator';

add(2, 3)           // returns 5
subtract(10, 4)     // returns 6
calculate("2+3*4")  // returns 20 (left to right: (2+3)*4)
formatNumber(3.14159265358979) // returns "3.1415926536"
```
