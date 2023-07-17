import sys
import itertools

SIZE = int(sys.argv[1])
OBJS = int(sys.argv[2])
INFILE = sys.argv[3]
OUTFILE = sys.argv[4]


def inverse(num, perm):
    if num == SIZE:
        return SIZE
    return perm.index(num)


def act(mat, perm):
    return [
        [inverse(mat[perm[i]][perm[j]], perm) for j in range(SIZE)] for i in range(SIZE)
    ]


# We can permute objects and non-objects separately
perms = [
    p1 + p2
    for p1 in itertools.permutations(range(OBJS))
    for p2 in itertools.permutations(range(OBJS, SIZE))
]

uniques = []

with open(INFILE) as file:
    for line in file:
        numbers = list(map(int, line.split()))
        mat = [numbers[i * SIZE : (i + 1) * SIZE] for i in range(SIZE)]
        if all(act(mat, p) >= mat for p in perms):
            uniques.append(mat)

uniques.sort()

numcats = 0

with open(OUTFILE, "w", encoding="utf-8") as file:
    numcats = 0
    for mat in uniques:
        numcats += 1
        file.write(str(mat) + "\n")

print(numcats)
