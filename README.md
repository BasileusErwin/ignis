# Ignis lang

## Roadmap

### Goals

- [ ] TypeScript-like syntax
- [ ] Compiled
- [ ] Static and strict typing
- [ ] Possibility of transpiling to Lua and compability with neovim
- [ ] Scripting capability
- [ ] Extensive standard library
    - [ ] A good library http
    - [ ] Test library
    - [ ] CLI library
    - [ ] IO
    - [ ] Memory management
    - [ ] Env
    - [ ] Regex
    - [ ] Math
    - [ ] Time
    - [ ] Primitive
- [ ] "Package" manager

### 07/28/2023

- [ ] Lexer
- [ ] Parser

```typescript
7 + (30 - 3) * 4
```

### 07/31/2023

- [X] Lexer
- [ ] Parser

```typescript
let value = 7 + (30 - 3) * 4;
let helloWorld = "Hello World";
```

### 08/06/2023

- [X] Lexer
- [X] Parser expressions
- [ ] Parser statements

```typescript
let value = 7 + (30 - 3) * 4;
let helloWorld = "Hello World";
```

### 08/06/2023

- [X] Lexer
- [X] Parser expressions
- [X] Parser statements
- [ ] Parser functions
- [ ] Interpreter

### 08/10/2023

- [X] Lexer
- [X] Parser expressions
- [X] Parser statements
- [X] Evaluator expression
- [X] Typing
- [ ] Parser functions
