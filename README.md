# n_puzzle

## mandatory

-   [x] You must provide a Makefile with the usual rules.
-   [x] Implement the A\* search algorithm (or one of its variants, you’re free to choose) to solve an N-puzzle
-   [x] You have to manage various puzzle sizes (3, 4, 5, 17, ...). The higher your program can go without dying a horrible, horrible death, the better.
-   [x] You have to manage both randomly determined states (of your own generation of course), and input files that specify a starting board, the format of which is described in the appendix.
-   [x] The cost associated with each transition is always 1.
-   [x] The user must be able to choose between at LEAST 3 (relevant) heuristic functions. The Manhattan-distance heuristic is mandatory, the other two are up to you.
-   [x] Show total number of states ever selected in the "opened" set (complexity in time)
-   [x] Show maximum number of states ever represented in memory at the same time during the search (complexity in size)
-   [x] Show number of moves required to transition from the initial state to the final state, according to the search
-   [x] Show the ordered sequence of states that make up the solution, according to the search
-   [x] The puzzle may be unsolvable, in which case you have to inform the user and exit

## bonus

-   [x] Configure g(x) and h(x) to run both the uniform-cost and greedy searches
-   [x] Visualization
-   [x] Non-square puzzles
-   [x] Line by line solver

## solvers

-   [x] A\*
-   [x] Greedy Search
-   [x] Uniform Cost Search
-   [x] IDA\*
-   [ ] LPA\*
-   [ ] SMA\*
-   [ ] ID-Dual\* or Bi-directional A\*
-   [ ] JPS+

## heuristics

-   [x] manhattan
-   [x] euclidean
-   [x] chebyshev
-   [x] manhattan with conflicts
-   [x] hamming
-   [x] inversion distance
-   [ ] walking distance
-   [ ] pattern databases
