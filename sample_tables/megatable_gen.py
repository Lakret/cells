"""
Generates JSON that can be used as a stress test for the cells demo.
"""
import json


def get_cell_id(col, row):
    return f"{col}{row:02d}"


def gen_megatable() -> dict[str, str]:
    inputs = dict()

    for col_code in range(ord("A"), ord("Z") + 1):
        for row in range(1, 51):
            col = chr(col_code)
            cell_id = get_cell_id(col, row)

            if col == "A":
                inputs[cell_id] = str(row)
            else:
                prev_col = chr(col_code - 1)
                prev_cell_id = get_cell_id(prev_col, row)

                inputs[cell_id] = f"= {prev_cell_id} * 1.25"

    return {"inputs": inputs}


if __name__ == "__main__":
    megatable = gen_megatable()
    with open("megatable.json", "w") as f:
        json.dump(megatable, f)
