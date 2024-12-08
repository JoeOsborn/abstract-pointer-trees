# abstract-pointer-trees
abstract binding trees in c, with pointers

"Abstract binding trees" (see [PFPL](https://www.cs.cmu.edu/~rwh/pfpl/) Chapter 1.2) generalize abstract syntax trees with a notion of bound variables that respect renaming (alpha equivalence). The intuition is that the tree node representing a "use" of a variable points *up* the tree to its binding site.

This development attempts to interpret the idea with pointers. A variable is a pointer to null, which is freshly generated (malloc'd) when a lambda expression is created, and all uses of that variable refer to that pointer. Substitution redirects that pointer from null to the expression being substituted. Pointer expressions are evaluated by following the chain of redirects until they get to an expression.

Still TODO: 
* Deallocate memory. Can probably do reference counting to figure out when it is safe to do so.
* Parse surface syntax.
