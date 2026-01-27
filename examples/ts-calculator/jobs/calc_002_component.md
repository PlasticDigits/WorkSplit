---
context_files:
  - src/calculator.ts
output_dir: src/components/
output_file: Calculator.tsx
depends_on:
  - calc_001_logic
---

# Create Calculator React Component

## Overview
Create a beautiful, functional calculator React component using the calculator logic module.

## Requirements
- Modern, clean iOS-style calculator design
- Responsive layout using CSS Grid
- Full keyboard support
- Display shows current input and result
- Support for chaining operations

## Component Structure

```tsx
import { useState, useEffect, useCallback } from 'react';
import { add, subtract, multiply, divide, formatNumber, type Operator } from '../calculator';
import './Calculator.css';
```

## State Management
- `display`: Current display value (string)
- `previousValue`: Previously entered value for operations
- `operator`: Current operator awaiting second operand
- `waitingForOperand`: Whether we're waiting for the next number

## Button Layout (4x5 grid)

Row 1: C, +/-, %, ÷
Row 2: 7, 8, 9, ×
Row 3: 4, 5, 6, −
Row 4: 1, 2, 3, +
Row 5: 0 (span 2), ., =

## Features to Implement

1. **Digit Input**: Append digits to display, handle leading zeros
2. **Decimal Point**: Only allow one decimal per number
3. **Clear (C)**: Reset calculator to initial state
4. **Toggle Sign (+/-)**: Negate current display value
5. **Percent (%)**: Divide current value by 100
6. **Operators (+, -, ×, ÷)**: Store value and operator, wait for operand
7. **Equals (=)**: Perform calculation using stored values
8. **Keyboard Support**: Listen for keydown events
   - 0-9 for digits
   - +, -, *, / for operators
   - Enter or = for equals
   - Escape or c for clear
   - Backspace to delete last digit

## Keyboard Handler

Use `useEffect` to add a keydown listener on mount. Remove on cleanup.

## Component Export

```tsx
export default function Calculator() {
  // Implementation
}
```

## Styling Classes to Use

- `.calculator` - Main container
- `.display` - Display area at top
- `.buttons` - Grid container for buttons
- `.button` - Individual button
- `.button.operator` - Operator buttons (orange)
- `.button.function` - Function buttons (gray)
- `.button.zero` - Zero button (spans 2 columns)
- `.button.active` - Active operator highlight
