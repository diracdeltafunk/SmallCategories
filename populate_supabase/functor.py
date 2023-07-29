from category import *


class Functor:
    def __init__(
        self,
        domain: Category,
        codomain: Category,
        object_mapping: dict,
        morphism_mapping: dict,
    ):
        self.domain = domain
        self.codomain = codomain
        self.object_mapping = object_mapping
        self.morphism_mapping = morphism_mapping
        self.cache = {}

    def map_obj(self, o):
        return self.object_mapping.get(o)

    def map_mor(self, f):
        return self.morphism_mapping.get(f)

    def hom_mapping(self, x, y):
        return {f: self.morphism_mapping.get(f) for f in self.domain.hom(x, y)}


def validate(fun: Functor) -> bool:
    d = fun.domain
    c = fun.codomain
    # Ensure domains & codomains are respected
    for f in d.morphisms:
        g = fun.map_mor(f)
        if c.dom(g) != fun.map_obj(d.dom(f)):
            return False
        if c.cod(g) != fun.map_obj(d.cod(f)):
            return False
    # Ensure ids are mapped to ids
    for o in d.objects:
        if fun.map_obj(d.i(o)) != c.i(fun.map_obj(o)):
            return False
    # Ensure composition is respected
    for (f, g), h in d.composition:
        if fun.map_mor(h) != c.comp(fun.map_mor(f), fun.map_mor(g)):
            return False
    return True
