## SmallCategories

The SmallCategories Project (name inspired by the [SmallGroups](https://docs.gap-system.org/pkg/smallgrp/doc/chap1.html) library in GAP) aims to build a useful database of (isomorphism classes of) small finite categories. Currently, the database supports categories with ≤6 morphisms.

This repository contains the code that generates the database.

# At a Glance

**Note.** More than this is known -- see [OEIS](https://oeis.org/A125696) and [Cruttwell-Leblanc](https://www.reluctantm.com/gcruttw/publications/ams2014CruttwellCountingFiniteCats.pdf).

| Morphisms ↓ / Objects → | 0 | 1    | 2   | 3   | 4  | 5 | 6 | Total |
|-------------------------|---|------|-----|-----|----|---|---|-------|
| 0                       | 1 | 0    | 0   | 0   | 0  | 0 | 0 | **1**     |
| 1                       | 0 | 1    | 0   | 0   | 0  | 0 | 0 | **1**     |
| 2                       | 0 | 2    | 1   | 0   | 0  | 0 | 0 | **3**     |
| 3                       | 0 | 7    | 3   | 1   | 0  | 0 | 0 | **11**    |
| 4                       | 0 | 35   | 16  | 3   | 1  | 0 | 0 | **55**    |
| 5                       | 0 | 228  | 77  | 20  | 3  | 1 | 0 | **329**   |
| 6                       | 0 | 2237 | 485 | 111 | 21 | 3 | 1 | **2858**  |

# TODO

* Build a website for navigating the database.