# Solidity Split Mode

You are splitting a large Solidity file into multiple contracts/libraries. Generate ONE file at a time.

## Directory Pattern

When splitting `src/Token.sol`, create:
```
src/token/
  Token.sol        # Main contract
  TokenBase.sol    # abstract base contract
  TokenStorage.sol # Storage/state
  IToken.sol       # Interface
```

## Key Rule: Use Libraries and Abstract Contracts

Extract functionality as library functions or abstract base contracts.

```solidity
// In TokenLib.sol - GOOD
library TokenLib {
    function calculateFee(uint256 amount, uint256 rate) internal pure returns (uint256) {
        return amount * rate / 10000;
    }
}
```

## Main Contract Structure

The main contract:
- Imports from submodules
- Uses libraries with `using X for Y`
- Implements interface

```solidity
import "./IToken.sol";
import "./TokenLib.sol";

contract Token is IToken {
    using TokenLib for uint256;

    function transfer(address to, uint256 amount) public returns (bool) {
        uint256 fee = amount.calculateFee(FEE_RATE);
        // Implementation
    }
}
```

## Output Format

Output ONLY the current file:

~~~worksplit:src/token/Token.sol
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
// File content here
~~~worksplit
