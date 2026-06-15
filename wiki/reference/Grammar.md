<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
-->

# JtV Grammar Reference

Complete EBNF grammar specification for Julia the Viper.

## Notation

```
::=     Definition
|       Alternative
[ ]     Optional (0 or 1)
{ }     Repetition (0 or more)
( )     Grouping
" "     Terminal string
' '     Terminal character
..      Range
```

## Lexical Grammar

### Whitespace and Comments

```ebnf
WHITESPACE ::= ' ' | '\t' | '\n' | '\r'
LINE_COMMENT ::= "//" {ANY_CHAR} NEWLINE
BLOCK_COMMENT ::= "/*" {ANY_CHAR} "*/"
```

### Identifiers

```ebnf
IDENTIFIER ::= (LETTER | '_') {LETTER | DIGIT | '_'}
LETTER ::= 'a'..'z' | 'A'..'Z'
DIGIT ::= '0'..'9'
```

### Keywords

```ebnf
KEYWORD ::= "if" | "else" | "while" | "for" | "in"
          | "fn" | "return" | "let" | "mut"
          | "true" | "false" | "print"
          | "reverse" | "struct" | "enum"
          | "match" | "import" | "module"
```

### Literals

#### Integer Literals

```ebnf
INTEGER ::= DECIMAL | HEXADECIMAL | BINARY | OCTAL
DECIMAL ::= ['-'] DIGIT {DIGIT}
HEXADECIMAL ::= "0x" HEX_DIGIT {HEX_DIGIT}
BINARY ::= "0b" BIN_DIGIT {BIN_DIGIT}
OCTAL ::= "0o" OCT_DIGIT {OCT_DIGIT}
HEX_DIGIT ::= DIGIT | 'a'..'f' | 'A'..'F'
BIN_DIGIT ::= '0' | '1'
OCT_DIGIT ::= '0'..'7'
```

#### Float Literals

```ebnf
FLOAT ::= ['-'] DIGIT {DIGIT} '.' DIGIT {DIGIT} [EXPONENT]
EXPONENT ::= ('e' | 'E') ['+' | '-'] DIGIT {DIGIT}
```

#### Rational Literals

```ebnf
RATIONAL ::= INTEGER '/' POSITIVE_INTEGER
POSITIVE_INTEGER ::= DIGIT {DIGIT}  (* non-zero *)
```

#### Complex Literals

```ebnf
COMPLEX ::= [REAL_PART] IMAGINARY_PART
REAL_PART ::= INTEGER | FLOAT
IMAGINARY_PART ::= (INTEGER | FLOAT) 'i'
```

#### String Literals

```ebnf
STRING ::= '"' {STRING_CHAR | ESCAPE_SEQ} '"'
STRING_CHAR ::= ANY_CHAR - ('"' | '\' | NEWLINE)
ESCAPE_SEQ ::= '\' ('n' | 't' | 'r' | '\' | '"' | '0' | 'x' HEX_DIGIT HEX_DIGIT)
```

### Operators

```ebnf
OPERATOR ::= '+' | '=' | '==' | '!=' | '<' | '>' | '<=' | '>='
           | '&&' | '||' | '!' | '..' | '+=' | '-='
```

### Delimiters

```ebnf
DELIMITER ::= '(' | ')' | '{' | '}' | '[' | ']'
            | ',' | ':' | ';' | '->'
```

## Syntactic Grammar

### Program Structure

```ebnf
Program ::= {TopLevelItem}
TopLevelItem ::= FunctionDef | StructDef | Import | Statement
```

### Imports

```ebnf
Import ::= "import" ModulePath
ModulePath ::= IDENTIFIER {'.' IDENTIFIER}
```

### Function Definitions

```ebnf
FunctionDef ::= [PurityAnnotation] "fn" IDENTIFIER '(' [Parameters] ')' [':' Type] Block

PurityAnnotation ::= "@pure" | "@total"

Parameters ::= Parameter {',' Parameter}
Parameter ::= IDENTIFIER ':' Type

Block ::= '{' {Statement} '}'
```

### Struct Definitions

```ebnf
StructDef ::= "struct" IDENTIFIER '{' {FieldDef} '}'
FieldDef ::= IDENTIFIER ':' Type ','
```

### Statements (Control Language)

```ebnf
Statement ::= Assignment
            | IfStatement
            | WhileStatement
            | ForStatement
            | ReturnStatement
            | PrintStatement
            | ReverseBlock
            | ExprStatement
            | Block

Assignment ::= IDENTIFIER '=' DataExpr
             | IDENTIFIER '+=' DataExpr
             | IDENTIFIER '-=' DataExpr

IfStatement ::= "if" Condition Block ["else" (Block | IfStatement)]

WhileStatement ::= "while" Condition Block

ForStatement ::= "for" IDENTIFIER "in" Range Block
Range ::= DataExpr ".." DataExpr

ReturnStatement ::= "return" [DataExpr]

PrintStatement ::= "print" '(' DataExpr ')'

ReverseBlock ::= "reverse" '{' {ReversibleStatement} '}'
ReversibleStatement ::= IDENTIFIER '+=' DataExpr
                      | IDENTIFIER '-=' DataExpr

ExprStatement ::= DataExpr
```

### Conditions

```ebnf
Condition ::= DataExpr CompareOp DataExpr
            | DataExpr
            | '!' Condition
            | Condition '&&' Condition
            | Condition '||' Condition
            | '(' Condition ')'

CompareOp ::= '==' | '!=' | '<' | '>' | '<=' | '>='
```

### Data Expressions (Data Language)

```ebnf
DataExpr ::= Term {'+' Term}
Term ::= Factor
Factor ::= Literal
         | IDENTIFIER
         | FunctionCall
         | '(' DataExpr ')'
         | '-' Factor

Literal ::= INTEGER | FLOAT | RATIONAL | COMPLEX | STRING | "true" | "false"

FunctionCall ::= IDENTIFIER '(' [Arguments] ')'
Arguments ::= DataExpr {',' DataExpr}
```

### Types

```ebnf
Type ::= PrimitiveType | CompoundType | FunctionType

PrimitiveType ::= "Int" | "Float" | "Rational" | "Complex"
                | "String" | "Bool" | "Unit"
                | "Hex" | "Binary"

CompoundType ::= ArrayType | TupleType | StructType
ArrayType ::= '[' Type ']'
TupleType ::= '(' Type {',' Type} ')'
StructType ::= IDENTIFIER

FunctionType ::= '(' [TypeList] ')' '->' Type
TypeList ::= Type {',' Type}
```

## Precedence and Associativity

| Precedence | Operator | Associativity |
|------------|----------|---------------|
| 1 (lowest) | `\|\|` | Left |
| 2 | `&&` | Left |
| 3 | `==` `!=` | Left |
| 4 | `<` `>` `<=` `>=` | Left |
| 5 | `+` | Left |
| 6 | Unary `-` `!` | Right |
| 7 (highest) | Function call | Left |

## Grammar Constraints

### Harvard Architecture Constraints

1. **Data expressions cannot contain control flow**:
   ```
   DataExpr ∩ (IfStatement | WhileStatement | ForStatement) = ∅
   ```

2. **Data expressions can only call pure/total functions**:
   ```
   FunctionCall in DataExpr ⟹ callee.purity ∈ {Total, Pure}
   ```

3. **Reverse blocks only allow reversible operations**:
   ```
   ReversibleStatement ::= IDENTIFIER '+=' DataExpr
                        | IDENTIFIER '-=' DataExpr
   ```

### Well-Formedness Rules

1. **Rational denominators must be non-zero**:
   ```
   RATIONAL = a/b ⟹ b ≠ 0
   ```

2. **Function names must be unique in scope**:
   ```
   ∀ f₁, f₂ in Scope: f₁.name = f₂.name ⟹ f₁ = f₂
   ```

3. **Variables must be defined before use**:
   ```
   use(x) ⟹ ∃ def(x) preceding use(x)
   ```

4. **@total functions cannot contain loops**:
   ```
   @total fn f { body } ⟹ body ∩ (While | For) = ∅
   ```

5. **@pure functions cannot perform I/O**:
   ```
   @pure fn f { body } ⟹ body ∩ (Print | Read) = ∅
   ```

## Example Parse Trees

### Simple Assignment

```
x = 5 + 3

Program
└── Statement
    └── Assignment
        ├── IDENTIFIER: "x"
        ├── '='
        └── DataExpr
            ├── Term
            │   └── Factor
            │       └── INTEGER: 5
            ├── '+'
            └── Term
                └── Factor
                    └── INTEGER: 3
```

### Function Definition

```
@pure fn multiply(a: Int, b: Int): Int {
    result = 0
    for i in 0..b {
        result = result + a
    }
    return result
}

Program
└── FunctionDef
    ├── PurityAnnotation: "@pure"
    ├── "fn"
    ├── IDENTIFIER: "multiply"
    ├── Parameters
    │   ├── Parameter
    │   │   ├── IDENTIFIER: "a"
    │   │   └── Type: "Int"
    │   └── Parameter
    │       ├── IDENTIFIER: "b"
    │       └── Type: "Int"
    ├── ReturnType: "Int"
    └── Block
        ├── Assignment: result = 0
        ├── ForStatement
        │   ├── IDENTIFIER: "i"
        │   ├── Range: 0..b
        │   └── Block
        │       └── Assignment: result = result + a
        └── ReturnStatement
            └── DataExpr: result
```

## See Also

- [Syntax Overview](../language/Syntax-Overview.md)
- [Parser Implementation](../internals/Parser.md)
- [Lexer Implementation](../internals/Lexer.md)
