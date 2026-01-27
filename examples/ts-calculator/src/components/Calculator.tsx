import { useState, useEffect, useCallback } from 'react';
import { add, subtract, multiply, divide, formatNumber, type Operator } from '../calculator';
import './Calculator.css';

export default function Calculator() {
  const [display, setDisplay] = useState('0');
  const [previousValue, setPreviousValue] = useState<number | null>(null);
  const [operator, setOperator] = useState<Operator | null>(null);
  const [waitingForOperand, setWaitingForOperand] = useState(false);

  const handleDigit = useCallback((digit: string) => {
    if (waitingForOperand) {
      setDisplay(digit);
      setWaitingForOperand(false);
    } else {
      setDisplay(prev => {
        if (prev === '0') return digit;
        if (prev.length < 9) return prev + digit;
        return prev;
      });
    }
  }, [waitingForOperand]);

  const handleDecimal = useCallback(() => {
    if (waitingForOperand) {
      setDisplay('0.');
      setWaitingForOperand(false);
    } else {
      setDisplay(prev => {
        if (!prev.includes('.')) return prev + '.';
        return prev;
      });
    }
  }, [waitingForOperand]);

  const handleClear = useCallback(() => {
    setDisplay('0');
    setPreviousValue(null);
    setOperator(null);
    setWaitingForOperand(false);
  }, []);

  const handleToggleSign = useCallback(() => {
    const value = parseFloat(display);
    setDisplay(formatNumber(-value));
  }, [display]);

  const handlePercent = useCallback(() => {
    const value = parseFloat(display);
    setDisplay(formatNumber(value / 100));
  }, [display]);

  const handleOperator = useCallback((nextOperator: Operator) => {
    const inputValue = parseFloat(display);

    if (previousValue === null) {
      setPreviousValue(inputValue);
    } else if (operator) {
      const result = calculate(previousValue, inputValue, operator);
      setDisplay(formatNumber(result));
      setPreviousValue(result);
    }

    setWaitingForOperand(true);
    setOperator(nextOperator);
  }, [display, previousValue, operator]);

  const handleEquals = useCallback(() => {
    if (!operator || previousValue === null) return;

    const inputValue = parseFloat(display);
    const result = calculate(previousValue, inputValue, operator);
    setDisplay(formatNumber(result));
    setPreviousValue(null);
    setOperator(null);
    setWaitingForOperand(true);
  }, [display, previousValue, operator]);

  const handleBackspace = useCallback(() => {
    if (waitingForOperand) return;
    setDisplay(prev => {
      if (prev.length === 1) return '0';
      return prev.slice(0, -1);
    });
  }, [waitingForOperand]);

  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    const key = event.key;

    if (/^[0-9]$/.test(key)) {
      handleDigit(key);
    } else if (key === '.') {
      handleDecimal();
    } else if (key === 'Enter' || key === '=') {
      event.preventDefault();
      handleEquals();
    } else if (key === 'Escape' || key === 'c' || key === 'C') {
      handleClear();
    } else if (key === 'Backspace') {
      handleBackspace();
    } else if (['+', '-', '*', '/'].includes(key)) {
      let operator: Operator;
      if (key === '*') operator = '*';
      else if (key === '/') operator = '/';
      else operator = key as Operator;
      handleOperator(operator);
    }
  }, [handleDigit, handleDecimal, handleEquals, handleClear, handleBackspace, handleOperator]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

  const getButtonClass = (value: string, type: 'operator' | 'function' | 'number' = 'number') => {
    let className = 'button';
    if (value === '0') className += ' zero';
    if (type === 'operator') className += ' operator';
    if (type === 'function') className += ' function';
    return className;
  };

  const buttons = [
    ['C', '+/-', '%', '÷'],
    ['7', '8', '9', '×'],
    ['4', '5', '6', '−'],
    ['1', '2', '3', '+'],
    ['0', '.', '=']
  ];

  return (
    <div className="calculator">
      <div className="display">
        <div className="previous-value">{previousValue !== null && operator ? `${formatNumber(previousValue)} ${operator}` : ''}</div>
        <div className="current-value">{display}</div>
      </div>
      <div className="buttons">
        {buttons.map((row, rowIndex) => (
          <div key={rowIndex} className="button-row">
            {row.map((button) => {
              let type: 'operator' | 'function' | 'number' = 'number';
              if (['+', '-', '×', '÷', '='].includes(button)) type = 'operator';
              if (['C', '+/-', '%'].includes(button)) type = 'function';

              return (
                <button
                  key={button}
                  className={getButtonClass(button, type)}
                  onClick={() => {
                    if (button === 'C') handleClear();
                    else if (button === '+/-') handleToggleSign();
                    else if (button === '%') handlePercent();
                    else if (button === '.') handleDecimal();
                    else if (button === '=') handleEquals();
                    else if (['+', '-', '×', '÷'].includes(button)) {
                      let op: Operator;
                      if (button === '×') op = '*';
                      else if (button === '÷') op = '/';
                      else op = button as Operator;
                      handleOperator(op);
                    } else {
                      handleDigit(button);
                    }
                  }}
                >
                  {button}
                </button>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}

function calculate(prev: number, current: number, op: Operator): number {
  switch (op) {
    case '+': return add(prev, current);
    case '-': return subtract(prev, current);
    case '*': return multiply(prev, current);
    case '/': return divide(prev, current);
  }
}