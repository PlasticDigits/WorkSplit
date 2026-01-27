# Manager Instructions for Creating Job Files

This document explains how to create job files for WorkSplit when breaking down a Solidity/Foundry feature into implementable chunks.

## CRITICAL: When to Use WorkSplit vs Direct Editing

**WorkSplit has overhead** (job creation, validation, verification, retries). Only use it when the cost savings outweigh this overhead.

### Cost Decision Matrix

| Task Size | Lines Changed | Recommendation | Reason |
|-----------|---------------|----------------|--------|
| Tiny | < 20 lines | **Direct edit** | Job overhead far exceeds savings |
| Small | 20-100 lines | **Direct edit** | Still faster to edit directly |
| Medium | 100-300 lines | **Evaluate** | Break-even zone; use WorkSplit for complex logic |
| Large | 300-500 lines | **WorkSplit** | Clear cost savings from free Ollama tokens |
| Very Large | 500+ lines | **WorkSplit strongly** | Significant savings; split into multiple jobs |

### Quick Decision Guide

```
STOP - Before creating a WorkSplit job, ask:

1. Is this < 100 lines of changes?
   → YES: Edit directly, don't use WorkSplit
   
2. Is this a simple, surgical change?
   → YES: Edit directly, WorkSplit overhead not worth it
   
3. Will this generate 300+ lines of NEW code?
   → YES: Use WorkSplit, clear savings
   
4. Is the logic complex enough to benefit from verification?
   → YES: Use WorkSplit
   → NO: Edit directly
```

---

## Quick Job Creation with Templates

**Preferred method**: Use `worksplit new-job` to scaffold job files quickly:

```bash
# Replace mode - generate a new contract
worksplit new-job feature_001 --template replace -o src/ -f MyContract.sol

# Edit mode - modify existing contracts
worksplit new-job fix_001 --template edit --targets src/Token.sol

# With context files
worksplit new-job impl_001 --template replace -c src/interfaces/IToken.sol -o src/ -f Token.sol

# Split mode - break large contract into modules
worksplit new-job split_001 --template split --targets src/LargeContract.sol

# Sequential mode - multi-file with context accumulation
worksplit new-job big_001 --template sequential -o src/
```

After running, edit the generated `jobs/<name>.md` to add specific requirements.

### When to Use Each Template

| Template | Use When | Reliability |
|----------|----------|-------------|
| `replace` | Creating new contracts or completely rewriting existing ones | High |
| `edit` | Making 2-5 small, isolated changes to existing contracts | Medium |
| `split` | A contract exceeds 500 lines and needs to be modularized | High |
| `sequential` | Generating multiple interdependent contracts | High |
| `tdd` | You want Foundry tests generated before implementation | High |

## Job File Format

Each job file uses YAML frontmatter followed by markdown instructions:

```markdown
---
context_files:
  - src/interfaces/IToken.sol
  - src/libraries/SafeMath.sol
output_dir: src/
output_file: Token.sol
---

# Create ERC20 Token

## Requirements
- Implement ERC20 standard
- Add minting capability for owner
- Include pause functionality

## Functions to Implement
- `mint(address to, uint256 amount) external onlyOwner`
- `pause() external onlyOwner`
- `unpause() external onlyOwner`
```

## Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `context_files` | No | List of files to include as context (max 2, each under 1000 lines) |
| `output_dir` | Yes | Directory where the output file will be created |
| `output_file` | Yes | Name of the generated file (default if multi-file output is used) |
| `output_files` | No | List of files to generate in sequential mode |
| `sequential` | No | Enable sequential mode (one LLM call per file) |
| `mode` | No | Output mode: "replace" (default) or "edit" for surgical changes |
| `target_files` | No | Files to edit when using edit mode |

## Solidity-Specific Best Practices

### 1. Size Jobs Appropriately

Each job should generate **at most 900 lines of code**. Smart contracts should typically be:
- Single responsibility (one main purpose per contract)
- Interface + Implementation pattern
- Use inheritance to split large contracts

### 2. Choose Context Files Wisely

Context files should:
- Define interfaces the contract will implement
- Show library functions to use (OpenZeppelin, etc.)
- Contain base contracts to inherit from

### 3. Write Clear Instructions

Good instructions include:
- **What** to create (contract, interface, library)
- **Security requirements** (access control, reentrancy guards)
- **Events** to emit
- **Modifiers** to implement
- **Storage layout** considerations

### 4. Naming Convention

```
feature_order_component.md

Examples:
- token_001_interface.md
- token_002_base.md
- token_003_implementation.md
- token_004_tests.md
```

This ensures jobs run in dependency order (alphabetically).

### 5. Foundry Project Structure

Standard Foundry layout:
```
project/
├── src/                    # Contract source files
│   ├── interfaces/         # Interface definitions
│   ├── libraries/          # Library contracts
│   └── Token.sol           # Main contracts
├── test/                   # Foundry tests
│   └── Token.t.sol
├── script/                 # Deployment scripts
│   └── Deploy.s.sol
└── foundry.toml
```

## TDD Workflow

To enable Test-Driven Development with Foundry, add the `test_file` field:

```yaml
---
context_files: []
output_dir: src/
output_file: Token.sol
test_file: test/Token.t.sol
---
```

When `test_file` is specified:
1. Foundry tests are generated FIRST based on requirements
2. Implementation is then generated to pass tests
3. Implementation is verified against requirements
