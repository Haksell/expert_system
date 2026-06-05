# expert_system

## todo

ambiguous flag:
- add back
- can only be undetermined if or/xor in conclusion
- otherwise like before

more tests:
- testing of update
- better test with xor
- testing with false
- testing of parsing

## subject

parsing:
- [x] your program must accept one parameter: the input file
- [x] this file will contain a list of rules
- [x] this file will contain a list of initial facts
- [x] this file will contain a list of queries
- [x] a fact can be any uppercase alphabetical character
- [x] if there is a syntax error in the input, the program must notify the user of the issue
- [x] if there is a contradiction in the input, the program must notify the user of the issue

the engine must support the following features:
- [x] AND conditions. e.g. "If A and B and [...] then X".
- [x] OR conditions. e.g. "If C or D then Z".
- [x] XOR conditions. e.g. "If A xor E then V".
- [x] negation. e.g. "If A and not B then Y".
- [x] multiple rules with the same conclusion.
- [x] AND in conclusions. e.g. "If A then B and C".
- [x] parentheses in expressions. these should be interpreted similarly to how they
are used in arithmetic expressions.

you must implement a backward-chaining inference engine:
- [x] for each query, the program must determine whether the query is true, false or undetermined
- [x] a fact can only be undetermined if the ruleset is ambiguous (e.g. "A is true, and if A then B or C", then B and C are undetermined)

the order of operations is:
- [x] `()`, `!`, `+`=`&`, `|`, `^`, `=>`=`<=>`=`<=`

## bonus

- [x] interactive fact validation
- [x] multiple queries and facts in same file
- [x] OR and XOR in conclusions
- [x] equivalence (`<=>`) and reverse implication (`<=`)
- [ ] --ambiguity flag
- [ ] reasoning visualization
- ... (at least one more bonus for 125)

## notes

- the subject requires to parse `+` as the AND operator, but it is generally used for the OR operator. we support it but prefer the more natural `&` and `|` in our examples.
- all binary gates: https://gist.github.com/cky26/58b28f011d512de1620719517dd7c0d4
