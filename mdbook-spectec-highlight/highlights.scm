; highlights.scm

; =========================================================
; Keywords
; =========================================================

[
  "syntax" "var" "relation" "rule" "rulegroup"
  "dec" "def" "extern" "builtin" "tbl"
  "if" "debug"
] @keyword

(else_premise) @keyword
(epsilon_literal) @keyword

"--" @keyword
(else_premise) @keyword
(if_premise "if" @keyword)

; =========================================================
; Punctuation
; =========================================================

[ "(" "`(" "`{" "[" "`[" ] @punctuation.bracket.open
[ ")" "}" "]" ] @punctuation.bracket.close
[ "<" ">" ] @punctuation.bracket

[ ":" "," "|" "/" ] @punctuation.delimiter

; =========================================================
; Comments
; =========================================================

(comment) @comment
(block_comment) @comment

; =========================================================
; Operators / atoms
; =========================================================

(atom_escape) @operator
(atom_upid) @operator

(infixop) @operator
(relop) @operator

["?" "*" "=" "=>" "<-" "::" "++"] @operator

; =========================================================
; Functions
; =========================================================

(func_dec_def "$" @function name: (defid) @function)
(func_def     "$" @function name: (defid) @function)
(extern_dec_def "$" @function name: (defid) @function)
(builtin_dec_def "$" @function name: (defid) @function)
(table_dec_def "$" @function name: (defid) @function)
(table_def     "$" @function name: (defid) @function)
(call_exp      "$" @function.call name: (defid) @function.call)

; =========================================================
; Relations and rules
; =========================================================

(relation_def       name: (relid) @type.definition)
(extern_relation_def name: (relid) @type.definition)
(rule_def           relation_name: (relid) @type)
(rule_def           rule_name: (ruleids) @label)
(rulegroup_def      relation_name: (relid) @type)
(rulegroup_def      rule_name: (ruleids) @label)
(rule               relation_name: (relid) @type)
(rule               rule_name: (ruleids) @label)
(rule_premise       relation_name: (relid) @type)
(rule_not_premise   relation_name: (relid) @type)

; =========================================================
; Types (syntax declarations/definitions)
; =========================================================

(syntax_stmt name: (synid name: (varid) @type.definition))
(syntax_stmt (synid name: (varid) @type.definition))
(variable_def name: (varid) @variable)
(variable_def type: (plaintyp) @type)

(tparam (varid) @type.parameter)
(targ (plaintyp) @type)

; Plain types
(bool_type) @type.builtin
(nat_type)  @type.builtin
(int_type)  @type.builtin
(text_type) @type.builtin

(var_type   (varid) @type)
(generic_type name: (varid) @type)

; =========================================================
; Variables / identifiers in expressions
; =========================================================

((arg) @variable.parameter (#set! priority 110))
(fieldexp name: (fieldid) @variable.parameter)
(var_exp    (varid) @variable.member)
(var_premise name: (varid) @variable.member)

; Constructors (uppercase ids used as data constructors)
(atom_upid (atomid (bare_upid) @constructor))

; =========================================================
; Hints
; =========================================================

(hint "hint" @label)
(hint name: (hint_name) @attribute)

; =========================================================
; Literals
; =========================================================

(boolean_literal) @constant.builtin
(number_literal)  @number
(text_literal)    @string
(epsilon_literal) @constant.builtin
(hole_exp)        @string.special
