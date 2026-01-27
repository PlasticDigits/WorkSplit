---
context_files:
  - src/components/Calculator.tsx
output_dir: src/components/
output_file: Calculator.css
depends_on:
  - calc_002_component
---

# Create Calculator Styles

## Overview
Create beautiful, modern CSS styles for the calculator component. iOS-inspired design with smooth animations.

## Design Requirements
- Dark theme matching iOS calculator
- Smooth hover and active states
- Responsive sizing
- Clean, minimal aesthetic
- CSS Grid for button layout

## Color Palette

```css
/* Background */
--calc-bg: #000000;

/* Display */
--display-text: #ffffff;

/* Number buttons */
--btn-number-bg: #333333;
--btn-number-hover: #4a4a4a;

/* Operator buttons */
--btn-operator-bg: #ff9500;
--btn-operator-hover: #ffad33;
--btn-operator-active: #ffffff;

/* Function buttons (C, +/-, %) */
--btn-function-bg: #a5a5a5;
--btn-function-hover: #c5c5c5;
--btn-function-text: #000000;
```

## Layout Specifications

### Calculator Container
- Max width: 320px
- Border radius: 20px
- Padding: 20px
- Background: black
- Box shadow for depth

### Display
- Full width
- Right-aligned text
- Font size: 48px (scales down for long numbers)
- Font family: system UI, monospace fallback
- Minimum height: 80px
- Text overflow: ellipsis

### Button Grid
- 4 columns, auto rows
- Gap: 12px
- Use CSS Grid

### Buttons
- Aspect ratio: 1/1 (square) except zero
- Border radius: 50% (circular)
- Font size: 28px
- Font weight: 500
- Cursor: pointer
- Transition: all 0.15s ease

### Zero Button
- Grid column: span 2
- Border radius: 40px (pill shape)
- Text align: left with padding

## Animations

### Button Hover
- Slight brightness increase
- Scale: 1.02

### Button Active (pressed)
- Scale: 0.95
- Brightness decrease

### Operator Active State
When an operator is selected and waiting for operand:
- Background: white
- Text color: orange

## Responsive Behavior
- On mobile: full width with padding
- On desktop: centered with max-width

## Additional Effects
- Add subtle text shadow to display
- Smooth font-size transition for display
- Focus-visible outline for accessibility
