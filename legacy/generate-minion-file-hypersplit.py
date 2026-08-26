# Generate category.minion file

import sys

assert len(sys.argv) >= 6

SIZE = int(sys.argv[1])
OBJS = int(sys.argv[2])
NONID_IDEMPOTENTS = int(sys.argv[3])
NONIDEMP_ENDOS = int(sys.argv[4])
NONENDO_ISO_PAIRS = int(sys.argv[5])

assert OBJS >= 0
assert NONID_IDEMPOTENTS >= 0
assert NONIDEMP_ENDOS >= 0
assert NONENDO_ISO_PAIRS >= 0
assert OBJS + NONID_IDEMPOTENTS + NONIDEMP_ENDOS + 2 * NONENDO_ISO_PAIRS <= SIZE

if SIZE == 0:
    print(
        """MINION 3
**VARIABLES**
DISCRETE mat[0,0] {0..0}
**EOF**"""
    )
    sys.exit(0)

assert OBJS >= 1

print("MINION 3")

print("**VARIABLES**")
print("BOOL isdef[{},{}]".format(SIZE, SIZE))
print("DISCRETE mat[{},{}] {{0..{}}}".format(SIZE, SIZE, SIZE))
# * Unfortunately this doesn't parse. *
# print(
#     "ALIAS assocl[{},{},{}] = {}".format(
#         SIZE,
#         SIZE,
#         SIZE,
#         "[{}]".format(
#             ",".join(
#                 "[{}]".format(
#                     ",".join(
#                         "[{}]".format(
#                             ",".join(
#                                 "mat[mat[{},{}],{}]".format(i, j, k)
#                                 for k in range(SIZE)
#                             )
#                         )
#                         for j in range(SIZE)
#                     )
#                 )
#                 for i in range(SIZE)
#             )
#         ),
#     )
# )
# print(
#     "ALIAS assocr[{},{},{}] = {}".format(
#         SIZE,
#         SIZE,
#         SIZE,
#         "[{}]".format(
#             ",".join(
#                 "[{}]".format(
#                     ",".join(
#                         "[{}]".format(
#                             ",".join(
#                                 "mat[{},mat[{},{}]]".format(i, j, k)
#                                 for k in range(SIZE)
#                             )
#                         )
#                         for j in range(SIZE)
#                     )
#                 )
#                 for i in range(SIZE)
#             )
#         ),
#     )
# )

# Force morphisms beyond OBJS to not be identities
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
    # dom/cod are right/left identities
    print("watchelement([mat[_,{}]], cod[{}], {})".format(i, i, i))
    print("watchelement([mat[{},_]], dom[{}], {})".format(i, i, i))

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

            print("    eq(isdef[{},{}],0),".format(i, j))
            print("    eq(isdef[{},{}],0)".format(j, k))

            print("})")

# Declare that the first OBJS-many morphisms are identities
for i in range(OBJS):
    print("eq(dom[{}],{})".format(i, i))
    print("eq(cod[{}],{})".format(i, i))

# Next NONID_IDEMPOTENTS are idempotents
for i in range(OBJS, OBJS + NONID_IDEMPOTENTS):
    print("element(mat[{},_], {}, {})".format(i, i, i))

# Next NONIDEMP_ENDOS are endomorphisms that are not idempotent
for i in range(OBJS + NONID_IDEMPOTENTS, OBJS + NONID_IDEMPOTENTS + NONIDEMP_ENDOS):
    # endo
    print("eq(isdef[{},{}],1)".format(i, i))
    # not idempotent
    print("watched-or({")
    print("    ineq({},mat[{},{}],-1),".format(i, i, i))
    print("    ineq(mat[{},{}],{},-1)".format(i, i, i))
    print("})")

# Next NONENDO_ISOS pairs are isomorphism-pairs that are not endomorphisms
for i in range(
    OBJS + NONID_IDEMPOTENTS + NONIDEMP_ENDOS,
    OBJS + NONID_IDEMPOTENTS + NONIDEMP_ENDOS + 2 * NONENDO_ISO_PAIRS,
    2,
):
    # i is not endo
    print("eq(isdef[{},{}],0)".format(i, i))
    # i+1 is inverse of i
    print("eq(mat[{},{}],cod[{}])".format(i, i + 1, i))
    print("eq(mat[{},{}],dom[{}])".format(i + 1, i, i))

# Rest are neither endos nor isos
for i in range(OBJS + NONID_IDEMPOTENTS + NONIDEMP_ENDOS + 2 * NONENDO_ISO_PAIRS, SIZE):
    # i is not endo
    print("eq(isdef[{},{}],0)".format(i, i))
    # i is not iso
    for j in range(SIZE):
        # j is not inverse of i, i.e.
        # i∘j ≠ cod(i) or j∘i ≠ dom(i)
        print("watched-or({")
        print("    ineq(cod[{}],mat[{},{}],-1),".format(i, i, j))
        print("    ineq(mat[{},{}],cod[{}],-1),".format(i, j, i))
        print("    ineq(dom[{}],mat[{},{}],-1),".format(i, j, i))
        print("    ineq(mat[{},{}],dom[{}],-1)".format(j, i, i))
        print("})")

print("**SEARCH**")

# Unclear if this is best. Probably worth testing all possible orders at some point.
print("VARORDER [dom,cod,isdef,mat]")
print("PRINT [mat]")

print("**EOF**")
