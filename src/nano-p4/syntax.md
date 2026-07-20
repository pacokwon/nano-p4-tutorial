# Nano-P4 Grammar

```
program
    : /* empty */
    | program declaration
    ;

declaration
    : instantiation
    | actionDeclaration
    | matchKindDeclaration
    | externDeclaration
    | parserDeclaration
    | controlDeclaration
    | typeDeclaration
    ;

typeDeclaration
    : derivedTypeDeclaration
    | parserTypeDeclaration
    | controlTypeDeclaration
    | packageTypeDeclaration
    ;

derivedTypeDeclaration
    : structTypeDeclaration
    | headerTypeDeclaration
    ;

structTypeDeclaration
    : STRUCT name "{" typeFieldList "}"
    ;

headerTypeDeclaration
    : HEADER name "{" typeFieldList "}"
    ;

typeFieldList
    : /* empty */
    | typeFieldList typeField
    ;

typeField
    : type name ";"
    ;

parserTypeDeclaration
    : PARSER name "(" parameterList ")" ";"
    ;

controlTypeDeclaration
    : CONTROL name "(" parameterList ")" ";"
    ;

packageTypeDeclaration
    : PACKAGE name "(" parameterList ")" ";"
    ;

instantiation
    : type "(" argumentList ")" name ";"
    ;

actionDeclaration
    : ACTION name "(" parameterList ")" blockStatement
    ;

matchKindDeclaration
    : MATCH_KIND "{" nameList "}"
    ;

externDeclaration
    : externObjectDeclaration
    ;

externObjectDeclaration
    : EXTERN name "{" externMethodPrototypeList "}"
    ;

externMethodPrototypeList
    : /* empty */
    | externMethodPrototypeList externMethodPrototype
    ;

externMethodPrototype
    : functionPrototype ";"
    ;

functionPrototype
    : VOID name "(" parameterList ")"
    ;

parserDeclaration
    : PARSER name "(" parameterList ")"
      "{" parserLocalDeclarationList parserStateList "}"
    ;

parserLocalDeclarationList
    : /* empty */
    | parserLocalDeclarationList parserLocalDeclaration
    ;

parserLocalDeclaration
    : variableDeclaration
    ;

parserStateList
    : parserState
    | parserStateList parserState
    ;

parserState
    : STATE name "{" statementList transitionStatement "}"
    ;

transitionStatement
    : TRANSITION stateExpression
    ;

stateExpression
    : name ";"
    | selectExpression
    ;

selectExpression
    : SELECT "(" expression ")" "{" selectCaseList "}"
    ;

selectCaseList
    : /* empty */
    | selectCaseList selectCase
    ;

selectCase
    : expression ":" name ";"
    ;

controlDeclaration
    : CONTROL name "(" parameterList ")"
      "{" controlLocalDeclarationList APPLY controlBody "}"
    ;

controlLocalDeclarationList
    : /* empty */
    | controlLocalDeclarationList controlLocalDeclaration
    ;

controlLocalDeclaration
    : variableDeclaration
    | tableDeclaration
    ;

controlBody
    : blockStatement
    ;

tableDeclaration
    : TABLE name "{" tableProperties "}"
    ;

tableProperties
    : tableKeyProperty tableActionsProperty
    | tableKeyProperty tableActionsProperty tableEntriesProperty
    ;

tableKeyProperty
    : KEY "=" tableKey
    ;

tableKey
    : "{" expression ":" name ";" "}"
    ;

tableActionsProperty
    : ACTIONS "=" "{" tableActionList "}"
    ;

tableActionList
    : tableAction
    | tableActionList tableAction
    ;

tableAction
    : tableActionReference ";"
    ;

tableActionReference
    : name
    | name "(" argumentList ")"
    ;

tableEntriesProperty
    : CONST ENTRIES "=" "{" tableEntryList "}"
    ;

tableEntryList
    : /* empty */
    | tableEntryList tableEntry
    ;

tableEntry
    : "(" expression ")" ":" tableActionReference ";"
    ;

statementList
    : /* empty */
    | statementList statement
    ;

statement
    : emptyStatement
    | variableDeclaration
    | assignmentStatement
    | callStatement
    | blockStatement
    | conditionalStatement
    ;

emptyStatement
    : ";"
    ;

variableDeclaration
    : type name "=" expression ";"
    ;

assignmentStatement
    : lvalue "=" expression ";"
    ;

callStatement
    : lvalue "(" argumentList ")" ";"
    ;

blockStatement
    : "{" statementList "}"
    ;

conditionalStatement
    : IF "(" expression ")" blockStatement ELSE blockStatement
    ;

lvalue
    : name
    | lvalue "." member
    | "(" lvalue ")"
    ;

expression
    : booleanLiteral
    | integerLiteral
    | name
    | unop expression
    | expression binop expression
    | expression "." member
    | name "(" argumentList ")"
    | "(" expression ")"
    ;

booleanLiteral
    : TRUE
    | FALSE
    ;

integerLiteral
    : INTEGER
    ;

name
    : IDENTIFIER
    | APPLY
    | KEY
    | ACTIONS
    | STATE
    ;

unop
    : "!"
    | "~"
    | "-"
    | "+"
    ;

binop
    : "*" | "+" | "-" | "/" | "%"
    | "<<" | ">>"
    | "<=" | ">=" | "<" | ">" | "!=" | "=="
    | "&" | "^" | "|"
    | "&&" | "||"
    ;

argumentList
    : /* empty */
    | argumentListNonEmpty
    ;

argumentListNonEmpty
    : expression
    | argumentListNonEmpty "," expression
    ;

parameterList
    : /* empty */
    | nonEmptyParameterList
    ;

nonEmptyParameterList
    : parameter
    | nonEmptyParameterList "," parameter
    ;

parameter
    : direction type name
    ;

direction
    : /* empty */
    | IN
    | OUT
    | INOUT
    ;

type
    : baseType
    | typeName
    ;

baseType
    : BOOL
    | MATCH_KIND
    | BIT "<" INTEGER ">"
    | INT "<" INTEGER ">"
    ;

typeName
    : TYPE_IDENTIFIER
    ;

member
    : name
    ;

nameList
    : name
    | nameList "," name
    ;
```
