/**
 * Supported arithmetic operators
 */
export type Operator = '+' | '-' | '*' | '/';

/**
 * Represents the state of the calculator
 */
export interface CalculatorState {
  display: string;
  previousValue: number | null;
  operator: Operator | null;
  waitingForOperand: boolean;
}

/**
 * Adds two numbers and returns their sum
 * @param a First number
 * @param b Second number
 * @returns The sum of a and b
 */
export function add(a: number, b: number): number {
  return a + b;
}

/**
 * Subtracts b from a and returns the difference
 * @param a First number
 * @param b Second number
 * @returns The difference of a and b
 */
export function subtract(a: number, b: number): number {
  return a - b;
}

/**
 * Multiplies two numbers and returns their product
 * @param a First number
 * @param b Second number
 * @returns The product of a and b
 */
export function multiply(a: number, b: number): number {
  return a * b;
}

/**
 * Divides a by b and returns the quotient
 * @param a First number
 * @param b Second number
 * @returns The quotient of a and b
 * @throws Error if b is 0
 */
export function divide(a: number, b: number): number {
  if (b === 0) {
    throw new Error('Division by zero');
  }
  return a / b;
}

/**
 * Parses and evaluates a simple arithmetic expression
 * @param expression The arithmetic expression string
 * @returns The result of the evaluation, or NaN for invalid expressions
 */
export function calculate(expression: string): number {
  if (!expression || !expression.trim()) {
    return NaN;
  }

  const trimmed = expression.trim();
  
  if (!/^[0-9+\-*/().\s]*$/.test(trimmed)) {
    return NaN;
  }

  try {
    const operators: Operator[] = [];
    const numbers: number[] = [];
    let currentNumber = '';

    for (let i = 0; i < trimmed.length; i++) {
      const char = trimmed[i];
      
      if (/\d/.test(char) || char === '.' || char === '-') {
        currentNumber += char;
      } else if (['+', '-', '*', '/'].includes(char as Operator)) {
        if (currentNumber !== '') {
          const num = parseFloat(currentNumber);
          if (isNaN(num)) {
            return NaN;
          }
          numbers.push(num);
          currentNumber = '';
        }
        
        operators.push(char as Operator);
      }
    }

    if (currentNumber !== '') {
      const num = parseFloat(currentNumber);
      if (isNaN(num)) {
        return NaN;
      }
      numbers.push(num);
    }

    if (numbers.length === 0) {
      return NaN;
    }

    let result = numbers[0];
    for (let i = 0; i < operators.length; i++) {
      const operator = operators[i];
      const nextNumber = numbers[i + 1];
      
      switch (operator) {
        case '+':
          result = add(result, nextNumber);
          break;
        case '-':
          result = subtract(result, nextNumber);
          break;
        case '*':
          result = multiply(result, nextNumber);
          break;
        case '/':
          result = divide(result, nextNumber);
          break;
      }
    }

    return result;
  } catch {
    return NaN;
  }
}

/**
 * Formats a number for display
 * @param num The number to format
 * @returns Formatted string representation
 */
export function formatNumber(num: number): string {
  if (isNaN(num) || !isFinite(num)) {
    return '0';
  }

  if (Math.abs(num) < 1e-10 || Math.abs(num) > 1e10) {
    return num.toExponential(10);
  }

  const rounded = Math.round(num * 1e10) / 1e10;
  
  const formatted = rounded.toString();
  const decimalIndex = formatted.indexOf('.');
  
  if (decimalIndex !== -1) {
    const decimalPart = formatted.slice(decimalIndex + 1);
    const nonZeroIndex = decimalPart.search(/[^0]/);
    
    if (nonZeroIndex === -1) {
      return formatted.slice(0, decimalIndex);
    }
    
    return formatted.slice(0, decimalIndex + nonZeroIndex + 1);
  }
  
  return formatted;
}