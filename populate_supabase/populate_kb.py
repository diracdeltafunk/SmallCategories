import os

import psycopg
from psycopg.rows import dict_row

from tqdm import tqdm

from category import *

BATCH_SIZE = 1000

IMPLICATIONS = [
    # Monoids are connected and not empty
    {
        "if_true": ["is_monoid"],
        "if_false": [],
        "then_true": ["is_connected"],
        "then_false": ["is_empty"],
    },
    {
        "if_true": ["has_terminal_object", "has_binary_products"],
        "if_false": [],
        "then_true": ["has_finite_products"],
        "then_false": [],
    },
    {
        "if_true": ["has_finite_products"],
        "if_false": [],
        "then_true": ["has_terminal_object", "has_binary_products"],
        "then_false": [],
    },
]


def prop_name_to_id(cur: psycopg.Cursor, prop_name: str) -> str | None:
    q = cur.execute(
        """
        SELECT
            id
        FROM
            "Propositions"
        WHERE
            "name" = '{}'
        ;
        """.format(
            prop_name
        )
    )
    if q.rowcount == 0:
        return None
    if q.rowcount == 1:
        return q.fetchone()["id"]
    raise Exception(
        "[prop_name_to_id] Unexpected count returned from Propositions: {}.".format(
            q.rowcount
        )
    )


def ensure_prop(cur: psycopg.Cursor, cat_id: str, prop_id: str, value: str) -> None:
    # Check if KB record exists
    check = cur.execute(
        """
        SELECT
          entry,value
        FROM
          "KnowledgeBase"
        WHERE
          "category" = '{}' AND
          "proposition" = '{}'
        ;
        """.format(
            cat_id, prop_id
        )
    )
    num_records = check.rowcount
    if num_records == 0:
        # Make new KB record
        cur.execute(
            """
            INSERT INTO "KnowledgeBase" ("category","proposition","value")
            VALUES ('{}','{}','{}')
            ;
            """.format(
                cat_id, prop_id, value
            )
        )
        return
    if num_records == 1:
        record = check.fetchone()
        if record["value"] == value:
            # No need to update
            return
        else:
            print(
                "WARNING: KB record for {} and {} has different value than desired. Updating this record.".format(
                    cat_id, prop_id
                )
            )
            # Update existing KB record
            cur.execute(
                """
                UPDATE "KnowledgeBase"
                SET "value" = '{}'
                WHERE
                  "entry" = '{}'
                ;
                """.format(
                    value, record["entry"]
                )
            )
        return
    if num_records > 1:
        print(
            "WARNING: Multiple KB records for {} and {}. Deleting and making a new one.".format(
                cat_id, prop_id
            )
        )
        # Delete KB records
        cur.execute(
            """
            DELETE FROM "KnowledgeBase"
            WHERE
              "category" = '{}' AND
              "proposition" = '{}'
            ;
            """.format(
                cat_id, prop_id
            )
        )
        # Make a new one
        cur.execute(
            """
            INSERT INTO "KnowledgeBase" ("category","proposition","value")
            VALUES ('{}','{}','{}')
            ;
            """.format(
                cat_id, prop_id, value
            )
        )
        return
    raise Exception(
        "Unexpected count returned from KnowledgeBase: {}.".format(num_records)
    )


def ensure_prop_name(
    cur: psycopg.Cursor, cat_id: str, prop_name: str, value: str
) -> None:
    prop_id = prop_name_to_id(cur, prop_name)
    if prop_id is None:
        raise Exception(
            "Proposition '{}' does not exist in the database.".format(prop_name)
        )
    ensure_prop(cur, cat_id, prop_id, value)


def ensure_impl(cur: psycopg.Cursor, cat_id: str, impl: dict[str, list[str]]) -> None:
    for id in impl["if_true"]:
        if not cur.execute(
            """
            SELECT EXISTS (
                SELECT 1
                FROM "KnowledgeBase"
                WHERE
                    "category" = '{}' AND
                    "proposition" = '{}' AND
                    "value" = TRUE
            ) AS "exists"
            """
        ).fetchone()["exists"]:
            # Precondition not satisfied.
            return
    for id in impl["if_false"]:
        if not cur.execute(
            """
            SELECT EXISTS (
                SELECT 1
                FROM "KnowledgeBase"
                WHERE
                    "category" = '{}' AND
                    "proposition" = '{}' AND
                    "value" = FALSE
            ) AS "exists"
            """
        ).fetchone()["exists"]:
            # Precondition not satisfied.
            return
    for id in impl["then_true"]:
        ensure_prop(cur, cat_id, id, True)
    for id in impl["then_false"]:
        ensure_prop(cur, cat_id, id, False)
    return


with psycopg.connect(
    "user=postgres password={} host=db.znxyuwheorjbdymlnrxe.supabase.co port=5432 dbname=postgres".format(
        os.environ.get("SUPABASE_PW")
    )
) as conn:
    conn.autocommit = True
    print("Connection successful!")
    with conn.cursor(row_factory=dict_row) as cur:
        numcats = cur.execute('SELECT COUNT(*) FROM "Categories";').fetchone()["count"]
        batches = numcats // BATCH_SIZE
        print(
            "There are {} categories. Using batch size {}.".format(numcats, BATCH_SIZE)
        )
        for i in tqdm(range(batches)):
            batch = cur.execute(
                'SELECT * FROM "Categories" LIMIT {} OFFSET {};'.format(
                    BATCH_SIZE, i * BATCH_SIZE
                )
            ).fetchall()
            for row in tqdm(batch, leave=False):
                cat = from_minion_matrix(row["table"])
                op = cat.op()
                pairs = [
                    (
                        "0ec2037a-4087-4398-ac1b-b0002a75ddda",
                        row["morphisms"] == 0,
                    ),  # is_initial
                    (
                        "f0fa8980-6c49-4078-8eef-8c4c9512b442",
                        row["morphisms"] == 1,
                    ),  # is_terminal
                    (
                        "c35a969b-8a67-4e49-85f8-52fe04c9cd9c",
                        cat.is_discrete(),
                    ),  # is_discrete
                    (
                        "45f7883d-e570-42d0-99a5-4499d246a8d9",
                        row["objects"] == 1,
                    ),  # is_monoid
                    (
                        "0cb83176-255e-49de-8997-33928e870818",
                        cat.has_terminal_object(),
                    ),  # has_terminal_object
                    (
                        "8a69155d-13e8-4368-afae-ea49f9d5c4f1",
                        op.has_terminal_object(),
                    ),  # has_initial_object
                    (
                        "c73a6041-0bf1-4418-ad47-26dcf434db69",
                        cat.is_preorder(),
                    ),  # is_preorder
                    (
                        "83e07597-92b5-4281-a9fd-b08f08f9bcaf",
                        cat.is_skeletal(),
                    ),  # is_skeletal
                    (
                        "4eb6a458-b137-4a97-bb26-439276541174",
                        cat.is_connected(),
                    ),  # is_connected
                    (
                        "3ec9d656-687e-40e2-901c-8bd62f7474b7",
                        cat.has_equalizers(),
                    ),  # has_equalizers
                    (
                        "43162acc-1023-4f79-b493-2d043f7c1db6",
                        op.has_equalizers(),
                    ),  # has_coequalizers
                    (
                        "608c3d0b-39fc-44b6-a6f6-d22b67e56d8b",
                        cat.is_groupoid(),
                    ),  # is_groupoid
                    (
                        "24a6c2f9-b7f9-44dc-875b-5369a5bc841c",
                        cat.has_binary_products(),
                    ),  # has_binary_products
                    (
                        "3687fed0-5471-4bee-84ff-097afcb244a8",
                        op.has_binary_products(),
                    ),  # has_binary_coproducts
                    (
                        "1a52ca8e-7c83-42bd-9879-552625586589",
                        cat.has_finite_products(),
                    ),  # has_finite_products
                    (
                        "e226f70b-5780-4c88-8178-85159e9c9145",
                        op.has_finite_products(),
                    ),  # has_finite_coproducts
                    (
                        "a581b1fc-5caf-4e67-baab-a457531190e8",
                        cat.is_finitely_complete(),
                    ),  # is_complete
                    (
                        "3b1b76be-f2cb-44d0-9b12-a91f347af7b7",
                        op.is_finitely_complete(),
                    ),  # is_cocomplete
                ]
                extant_props = cur.execute(
                    """
                    SELECT "proposition","value"
                    FROM "KnowledgeBase"
                    WHERE "category" = '{}'
                    ;
                    """.format(
                        row["id"]
                    )
                ).fetchall()
                for pair in pairs:
                    if any(
                        entry["proposition"] == pair[0] and entry["value"] == pair[1]
                        for entry in extant_props
                    ):
                        continue
                    ensure_prop(cur, row["id"], pair[0], pair[1])
