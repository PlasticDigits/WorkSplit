# Solidity Test Generation

You are generating Foundry tests - the implementation does not exist yet.

## Guidelines

- Use Foundry test patterns with `forge-std/Test.sol`
- Cover main functionality, edge cases, and revert conditions
- Use `vm.prank`, `vm.deal`, `vm.expectRevert` as needed

## Output Format

~~~worksplit
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/Contract.sol";

contract ContractTest is Test {
    Contract public target;

    function setUp() public {
        target = new Contract();
    }

    function test_functionName() public {
        uint256 result = target.functionName(input);
        assertEq(result, expected);
    }

    function test_revertCase() public {
        vm.expectRevert();
        target.functionName(badInput);
    }
}
~~~worksplit

Output ONLY test code. No explanations.
