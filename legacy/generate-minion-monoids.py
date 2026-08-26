# Generate monoid.minion file

import sys

SIZE = int(sys.argv[1])
NONID_IDEMPOTENTS = int(sys.argv[2])

assert 1 + NONID_IDEMPOTENTS <= SIZE

print("MINION 3")

print("**VARIABLES**")
print("DISCRETE mat[{},{}] {{0..{}}}".format(SIZE, SIZE, SIZE - 1))

print("**CONSTRAINTS**")
# Associativity
for i in range(SIZE):
    for j in range(SIZE):
        for k in range(SIZE):
            print("watched-or({")
            for n in range(SIZE):
                print("    watched-and({")
                print(
                    "        watchelement([mat[{},_]], mat[{},{}], {}),".format(
                        i, j, k, n
                    )
                )
                print(
                    "        watchelement([mat[_,{}]], mat[{},{}], {})".format(
                        k, i, j, n
                    )
                )
                print("    })" + ("," if n < SIZE - 1 else ""))

            print("})")

# 0 is identity
for i in range(SIZE):
    print("element(mat[{},_], 0, {})".format(i, i))
    print("element(mat[_,{}], 0, {})".format(i, i))

# Next NONID_IDEMPOTENTS are idempotents
for i in range(1, 1 + NONID_IDEMPOTENTS):
    print("element(mat[{},_], {}, {})".format(i, i, i))

# Rest are not idempotents
for i in range(1 + NONID_IDEMPOTENTS, SIZE):
    print("watched-or({")
    print("    ineq({},mat[{},{}],-1),".format(i, i, i))
    print("    ineq(mat[{},{}],{},-1)".format(i, i, i))
    print("})")

print("**EOF**")
