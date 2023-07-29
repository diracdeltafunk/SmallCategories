class Category:
    def __init__(
        self,
        objects: list,
        morphisms: list,
        domain: dict,
        codomain: dict,
        id: dict,
        composition: dict,
    ):
        self.objects = objects
        self.morphisms = morphisms
        self.domain = domain
        self.codomain = codomain
        self.id = id
        self.composition = composition
        self.cache = {}

    def comp(self, f, g):
        return self.composition.get((f, g))

    def dom(self, f):
        return self.domain.get(f)

    def cod(self, f):
        return self.codomain.get(f)

    def i(self, o):
        return self.id.get(o)

    def hom(self, x, y):
        return [m for m in self.morphisms if self.dom(m) == x and self.cod(m) == y]

    def id_functor(self):
        return Functor(
            domain=self,
            codomain=self,
            object_mapping={o: o for o in self.objects},
            morphism_mapping={m: m for m in self.morphisms},
        )

    def is_discrete(self):
        if "is_discrete" in self.cache:
            return self.cache["is_discrete"]
        self.cache["is_discrete"] = len(self.objects) == len(self.morphisms)
        return self.cache["is_discrete"]

    def has_terminal_object(self):
        if "has_terminal_object" in self.cache:
            return self.cache["has_terminal_object"]
        for x in self.objects:
            is_terminal = True
            for y in self.objects:
                if len(self.hom(y, x)) != 1:
                    is_terminal = False
                    break
            if is_terminal:
                self.cache["has_terminal_object"] = True
                return True
        self.cache["has_terminal_object"] = False
        return False

    def is_preorder(self):
        if "is_preorder" in self.cache:
            return self.cache["is_preorder"]
        for x in self.objects:
            for y in self.objects:
                if len(self.hom(x, y)) > 1:
                    self.cache["is_preorder"] = False
                    return False
        self.cache["is_preorder"] = True
        return True

    def is_endo(self, f):
        return self.dom(f) == self.cod(f)

    def is_iso(self, f):
        for g in self.hom(self.codomain.get(f), self.domain.get(f)):
            if self.comp(f, g) == self.i(self.cod(f)) and self.comp(g, f) == self.i(
                self.dom(f)
            ):
                return True
        return False

    def is_skeletal(self):
        if "is_skeletal" in self.cache:
            return self.cache["is_skeletal"]
        for f in self.morphisms:
            if self.is_iso(f) and not self.is_endo(f):
                self.cache["is_skeletal"] = False
                return False
        self.cache["is_skeletal"] = True
        return True

    def is_product(self, obj1, obj2, proj1, proj2):
        if (
            self.cod(proj1) != obj1
            or self.cod(proj2) != obj2
            or self.dom(proj1) != self.dom(proj2)
        ):
            return False
        p = self.dom(proj1)
        for z in self.objects:
            pairlist = [
                (m1, m2) for m1 in self.hom(z, obj1) for m2 in self.hom(z, obj2)
            ]
            for h in self.hom(z, p):
                if not ((self.comp(proj1, h), self.comp(proj2, h)) in pairlist):
                    return False
                pairlist.remove((self.comp(proj1, h), self.comp(proj2, h)))
            if len(pairlist) != 0:
                return False
        return True

    def binary_prod_exists(self, obj1, obj2):
        for p in self.objects:
            for proj1 in self.hom(p, obj1):
                for proj2 in self.hom(p, obj2):
                    if self.is_product(obj1, obj2, proj1, proj2):
                        return True
        return False

    def has_binary_products(self):
        if "has_binary_products" in self.cache:
            return self.cache["has_binary_products"]
        for obj1 in self.objects:
            for obj2 in self.objects:
                if not self.binary_prod_exists(obj1, obj2):
                    self.cache["has_binary_products"] = False
                    return False
        self.cache["has_binary_products"] = True
        return True

    def is_equalizer(self, f, g, h):
        if (
            self.dom(f) != self.cod(h)
            or self.dom(g) != self.cod(h)
            or self.cod(f) != self.cod(g)
        ):
            return False
        if self.comp(f, h) != self.comp(g, h):
            return False
        d = self.dom(f)
        x = self.dom(h)
        for z in self.objects:
            for q in self.hom(z, d):
                if self.comp(q, f) == self.comp(q, g):
                    if sum(1 for p in self.hom(z, x) if self.comp(h, p) == q) != 1:
                        return False
        return True

    def has_equalizers(self):
        if "has_equalizers" in self.cache:
            return self.cache["has_equalizers"]
        for f in self.morphisms:
            for g in self.hom(self.dom(f), self.cod(f)):
                has_equalizer = False
                for h in self.morphisms:
                    if self.is_equalizer(f, g, h):
                        has_equalizer = True
                if not has_equalizer:
                    self.cache["has_equalizers"] = False
                    return False
        self.cache["has_equalizers"] = True
        return True

    def has_finite_products(self):
        if "has_finite_products" in self.cache:
            return self.cache["has_finite_products"]
        self.cache["has_finite_products"] = (
            self.has_binary_products() and self.has_equalizers()
        )
        return self.cache["has_finite_products"]

    def is_finitely_complete(self):
        if "is_finitely_complete" in self.cache:
            return self.cache["is_finitely_complete"]
        self.cache["is_finitely_complete"] = (
            self.has_finite_products() and self.has_equalizers()
        )
        return self.cache["is_finitely_complete"]

    def is_connected(self):
        if "is_connected" in self.cache:
            return self.cache["is_connected"]
        if len(self.objects) == 0:
            self.cache["is_connected"] = False
            return False
        unseen = set(self.objects)
        queue = {self.objects[0]}
        while len(queue) > 0:
            current = queue.pop()
            unseen.discard(current)
            for x in unseen:
                for f in self.morphisms:
                    if (self.dom(f), self.cod(f)) in [(x, current), (current, x)]:
                        queue.add(x)
        self.cache["is_connected"] = len(unseen) == 0
        return self.cache["is_connected"]

    def is_groupoid(self):
        if "is_groupoid" in self.cache:
            return self.cache["is_groupoid"]
        self.cache["is_groupoid"] = all(self.is_iso(f) for f in self.morphisms)
        return self.cache["is_groupoid"]

    def op(self):
        return Category(
            objects=self.objects,
            morphisms=self.morphisms,
            domain=self.codomain,
            codomain=self.domain,
            id=self.id,
            composition={(f, g): self.comp(g, f) for (f, g) in self.composition},
        )


def compute_num_objs(mat: list) -> int:
    morphs = len(mat)
    objs = 0
    # Test if morphism i is an identity
    for i in range(morphs):
        is_id = True
        for j in range(morphs):
            if (mat[i][j] != j and mat[i][j] != morphs) or (
                mat[j][i] != j and mat[j][i] != morphs
            ):
                is_id = False
                break
        if not is_id:
            break
        objs += 1
    return objs


def from_minion_matrix(mat: list, objs: int = None) -> Category:
    morphs = len(mat)
    if objs is None:
        objs = compute_num_objs(mat)
    objects = list(range(objs))
    morphisms = list(range(morphs))
    domain = {m: next(i for i in range(objs) if mat[m][i] == m) for m in range(morphs)}
    codomain = {
        m: next(i for i in range(objs) if mat[i][m] == m) for m in range(morphs)
    }
    id = {i: i for i in range(objs)}
    composition = {
        (i, j): mat[i][j]
        for i in range(morphs)
        for j in range(morphs)
        if mat[i][j] != morphs
    }
    return Category(
        objects=objects,
        morphisms=morphisms,
        domain=domain,
        codomain=codomain,
        id=id,
        composition=composition,
    )


# def ordinal(n: int):
#     morphs = [(i, j) for j in range(n) for i in range(j + 1)]
#     return Category(
#         objects=list(range(n)),
#         morphisms=morphs,
#         domain={m: m[0] for m in morphs},
#         codomain={m: m[1] for m in morphs},
#         id={i: (i, i) for i in range(n)},
#         composition={
#             ((i, j), (j, k)): (i, k)
#             for k in range(n)
#             for j in range(k + 1)
#             for i in range(j + 1)
#         },
#     )


# initial = Category(
#     objects=[],
#     morphisms=[],
#     domain={},
#     codomain={},
#     id={},
#     composition={}
# )

# terminal = Category(
#     objects=[0],
#     morphisms=["!"],
#     domain={"!": 0},
#     codomain={"!": 0},
#     id={0: "!"},
#     composition={("!", "!"): "!"}
# )

# universal_map = Functor(
#     domain=initial,
#     codomain=terminal,
#     object_mapping={},
#     morphism_mapping={}
# )
