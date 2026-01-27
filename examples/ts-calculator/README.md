# TypeScript Calculator Example

A React calculator app built using WorkSplit to generate the code with a local LLM.

## Overview

This example demonstrates how to use WorkSplit to build a complete React application:

1. **Initialize project** - Create a Vite React TypeScript app and run `worksplit init`
2. **Create job files** - Define what code should be generated
3. **Run WorkSplit** - Let the local LLM generate the code
4. **Build and run** - Verify the generated code works

## Generated Files

WorkSplit generated these files from the job definitions:

- `src/calculator.ts` - Calculator logic module with arithmetic operations
- `src/components/Calculator.tsx` - React calculator component with iOS-style design
- `src/components/Calculator.css` - Styles for the calculator
- `src/components/index.ts` - Component exports

## How This Example Was Created

### Step 1: Create Vite React Project

```bash
mkdir examples/ts-calculator
cd examples/ts-calculator
npm create vite@latest . -- --template react-ts
npm install
```

### Step 2: Initialize WorkSplit

```bash
worksplit init --lang typescript --model worksplit-coder-glm-4.7:32k
```

This creates:
- `jobs/` directory with system prompts
- `worksplit.toml` configuration file
- Example job files (which we delete)

### Step 3: Create Job Files

Three job files were created in the `jobs/` directory:

**calc_001_logic.md** - Generates `src/calculator.ts`
- Pure TypeScript functions for add, subtract, multiply, divide
- Expression parsing and formatting utilities
- Type definitions for calculator state

**calc_002_component.md** - Generates `src/components/Calculator.tsx`
- React functional component with hooks
- Keyboard support for number entry
- Clean iOS-style button layout
- Uses calculator.ts for logic

**calc_003_styles.md** - Generates `src/components/Calculator.css`
- iOS-inspired dark theme design
- CSS Grid layout for buttons
- Hover and active states
- Responsive design

### Step 4: Run WorkSplit

```bash
worksplit run --batch
```

Output:
```
=== Batch Run Summary ===
Processed: 3
Passed:    3
Failed:    0

Results:
  calc_001_logic [PASS] (171 lines)
  calc_002_component [PASS] (188 lines)
  calc_003_styles [PASS] (140 lines)
```

### Step 5: Wire Up the App

After WorkSplit generates the components, update `App.tsx` to use the Calculator:

```tsx
import Calculator from './components/Calculator'
import './App.css'

function App() {
  return (
    <div className="app">
      <h1>React Calculator</h1>
      <p className="subtitle">Built with WorkSplit + Vite + TypeScript</p>
      <Calculator />
    </div>
  )
}

export default App
```

### Step 6: Build and Run

```bash
npm run build   # Verify no TypeScript errors
npm run dev     # Start development server
```

## Job File Structure

Each job file uses YAML frontmatter to configure WorkSplit:

```yaml
---
context_files:
  - src/calculator.ts      # Files to include as context
output_dir: src/components/ # Where to write output
output_file: Calculator.tsx # Output filename
depends_on:
  - calc_001_logic         # Wait for this job first
---
```

The markdown body describes what code to generate.

## Key Learnings

1. **Dependencies matter** - Use `depends_on` so the component job can reference the logic module
2. **Be specific** - Include type signatures, function names, and expected behavior
3. **Context files** - Pass related files so the LLM understands the codebase
4. **Verification** - WorkSplit verifies generated code before marking pass
5. **Minor fixes** - Some TypeScript adjustments may be needed after generation

## Running the App

```bash
cd examples/ts-calculator
npm install
npm run dev
```

Open http://localhost:5173 to see the calculator.

## Project Structure

```
ts-calculator/
├── jobs/                    # WorkSplit job files
│   ├── _jobstatus.json     # Job status tracking
│   ├── _systemprompt_*.md  # System prompts for LLM
│   ├── calc_001_logic.md   # Job: calculator logic
│   ├── calc_002_component.md # Job: React component
│   └── calc_003_styles.md  # Job: CSS styles
├── src/
│   ├── calculator.ts       # Generated: logic module
│   ├── components/
│   │   ├── Calculator.tsx  # Generated: React component
│   │   ├── Calculator.css  # Generated: styles
│   │   └── index.ts        # Generated: exports
│   ├── App.tsx             # Modified: uses Calculator
│   ├── App.css             # Modified: app styles
│   ├── index.css           # Modified: global styles
│   └── main.tsx            # Vite entry point
├── worksplit.toml          # WorkSplit config
├── package.json            # npm dependencies
└── vite.config.ts          # Vite config
```
