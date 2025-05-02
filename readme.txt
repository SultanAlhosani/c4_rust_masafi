This is the repository that we will use to make a c4 to rust compiler

Functionailities Implemented
Return statements
arithmetic operations/exprssions (+,/,*,-)
Parenthesies in expressiosn
if statemenets if(expression)
while loops    while(expression)
comparison operators (==, !=, <, etc..)
variables      int x = 5;
multiple statements
type system (int, boolean etc..)
function defenition
self-hosting


Functionalities remaining to implement
memory management documentation

problems faced:
---- vm::tests::test_arithmetic stdout ----
Current token: Return
thread 'vm::tests::test_arithmetic' panicked at src\vm.rs:205:9:      
assertion `left == right` failed
  left: 20
 right: 14
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
ned to fix precedence 

Replaced the old loop-based expression parser with a hierarchical recursive descent implementation to properly respect operator precedence.

Changes include:
- `expression()` now calls `parse_expr()` to begin parsing.
- Introduced precedence layers:
  - `parse_expr()` → entry point
  - `parse_cmp()` handles ==, <, >
  - `parse_add_sub()` handles +, -
  - `parse_mul_div()` handles *, /, %
  - `parse_primary()` handles literals, variables, grouped expressions, and function calls.
- This resolves the previously failing test: `return 2 + 3 * 4;` which incorrectly returned 20 instead of 14.
- Ensured parenthesis grouping takes highest precedence.

All existing unit tests pass after this fix.
