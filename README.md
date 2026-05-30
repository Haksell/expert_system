# expert_system

## todo

- parsing
- testing of parsing

## subject

- [ ] you must implement a backward-chaining inference engine
- [ ] a fact can be any uppercase alphabetical character
- [ ] your program must accept one parameter: the input file
- [ ] this file will contain a list of rules
- [ ] this file will contain a list of initial facts
- [ ] this file will contain a list of queries
- [ ] for each query, the program must determine whether the query is true, false or undetermined
- [ ] by default, all facts are considered false and can only be made true through the initial facts statement or the application of a rule
- [ ] a fact can only be undetermined if the ruleset is ambiguous (e.g. "A is true, and if A then B or C", then B and C are undertermined)
- [ ] if there is an error in the input, for example a contradiction in the facts or a syntax error, the program must notify the user of the issue

the engine must support the following features:
- [ ] AND conditions. e.g. "If A and B and [...] then X".
- [ ] OR conditions. e.g. "If C or D then Z".
- [ ] XOR conditions. e.g. "If A xor E then V".
- [ ] negation. e.g. "If A and not B then Y".
- [ ] multiple rules with the same conclusion. For example, several rules can result
in the same fact as their conclusion.
- [ ] AND in conclusions. e.g. "If A then B and C".
- [ ] parentheses in expressions. these should be interpreted similarly to how they
are used in arithmetic expressions.

the order of operations is:
- [ ] `()`, `!`, `+` (or `&`), `|`, `^`, `=>`, `<=>`

## bonus

- [ ] interactive fact validation
- [ ] reasoning visualization
- [ ] OR and XOR in conclusions
- [ ] biconditional rules (`<=>`)
- ... (at least one more bonus for 125)

## notes

the subject requires to parse `+` as the AND operator, but it is generally used for the OR operator. we support it but prefer the more natural `&` and `|` in our examples.