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
memory management.

problems faced:
---- vm::tests::test_arithmetic stdout ----
Current token: Return
thread 'vm::tests::test_arithmetic' panicked at src\vm.rs:205:9:      
assertion `left == right` failed
  left: 20
 right: 14
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
ned to fix precedence 