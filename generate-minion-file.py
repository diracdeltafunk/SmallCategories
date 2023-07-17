# Generate category.minion file

import sys

SIZE = int(sys.argv[1])
OBJS = int(sys.argv[2])

print("MINION 3")

print("**VARIABLES**")
print("BOOL isdef[{},{}]".format(SIZE, SIZE))
print("DISCRETE mat[{},{}] {{0..{}}}".format(SIZE, SIZE, SIZE))
print("DISCRETE dom[{}] {{0..{}}}".format(SIZE, OBJS - 1))
print("DISCRETE cod[{}] {{0..{}}}".format(SIZE, OBJS - 1))

print("**CONSTRAINTS**")
# Make sure mat[i,j]<SIZE (i.e. composition is defined) iff dom[i]==cod[j]
for i in range(SIZE):
    for j in range(SIZE):
        print("reify(eq(dom[{}],cod[{}]),isdef[{},{}])".format(i, j, i, j))
        print("reify(ineq(mat[{},{}],{},-1),isdef[{},{}])".format(i, j, SIZE, i, j))

for i in range(SIZE):
    # dom and cod are idempotent
    print("watchelement(dom, dom[{}], dom[{}])".format(i, i))
    print("watchelement(cod, cod[{}], cod[{}])".format(i, i))
    # dom and cod are l/r identities
    print("watchelement([mat[_,{}]], cod[{}], {})".format(i, i, i))
    print("watchelement([mat[{},_]], dom[{}], {})".format(i, i, i))
    # NOTE: these conditions imply that, for example,
    # dom(cod(f)) = cod(f) and cod(dom(f)) = dom(f)
    # Proof: cod(f)∘f = f is defined, so dom(cod(f)) = cod(f)

# # Declare that dom(f∘g) = dom(g) whenever defined by:
# # watched-or({eq(mat[i,j],SIZE), watchelement(dom, mat[i,j], dom[j])})
# for i in range(SIZE):
#     for j in range(SIZE):
#         print("watched-or({")
#         print("    eq(mat[{},{}],{}),".format(i, j, SIZE))
#         print("    watchelement(dom, mat[{},{}], dom[{}])".format(i, j, j))
#         print("})")
# # NOTE: This is not necessary! Already implied.
# # Proof: Suppose f∘g defined. Then
# # (f∘g)∘(dom g) = f∘(g∘(dom g)) = f∘g defined,
# # so dom(f∘g) = cod(dom(g)) = dom(g).
# # Likewise, (cod f)∘(f∘g) = (cod f ∘ f)∘g = f∘g defined,
# # so cod(f) = dom(cod(f)) = cod(f∘g). QED

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
                print("    }),")

            print("    watchelement([isdef[{},_]], {}, 0),".format(i, j))
            print("    watchelement([isdef[{},_]], {}, 0)".format(j, k))

            print("})")

# Declare that the first OBJS-many morphisms are identities
for i in range(OBJS):
    print("element(dom, {}, {})".format(i, i))
    print("element(cod, {}, {})".format(i, i))

print("**EOF**")
