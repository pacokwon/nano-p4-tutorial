# Chapter 1

```spectec
relation Expr_ok:
  cursor typingContext |- expression : typedExpressionIR
  hint(input %0 %1 %2)
  hint(prose_in "typing" %2#", under context" %1 "at" %0)
  hint(prose_out %3)
```
