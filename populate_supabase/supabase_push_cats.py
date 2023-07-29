import os
import sys
from supabase import Client
from supabase.client import create_client

SIZE = int(sys.argv[1])
OBJS = int(sys.argv[2])
INFILE = sys.argv[3]

url = "https://znxyuwheorjbdymlnrxe.supabase.co"
key = os.environ.get("SUPABASE_KEY")
supabase = create_client(url, key)

with open(INFILE) as file:
    i = 0
    for line in file:
        print("Processing #{}".format(i))
        formatted_table = line.replace("[", "{").replace("]", "}")
        check = (
            supabase.table("Categories")
            .select("id")
            .eq("table", formatted_table)
            .execute()
        )
        if len(check.data) == 0:
            supabase.table("Categories").insert(
                {
                    "objects": OBJS,
                    "morphisms": SIZE,
                    "index": i,
                    "table": formatted_table,
                }
            ).execute()
        i += 1
