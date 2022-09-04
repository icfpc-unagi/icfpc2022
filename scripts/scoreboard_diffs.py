import json
from pathlib import Path
import pandas as pd

path = sorted(Path("/Users/tos/Dropbox/ICFPC2022/scoreboard").glob("*.json"))[-1]
print(path)
j = json.load(open("/Users/tos/Dropbox/ICFPC2022/scoreboard/20220904-121252.json"))

# ['user_id', 'team_name', 'results', 'total_cost', 'solved_problem_count']
# ['problem_id', 'problem_name', 'last_submitted_at', 'submission_count', 'min_cost', 'overall_best_cost']

# print(j["users"][0]["results"][0].keys())

def results_to_dict(results):
    d = {}
    for r in results:
        k = r["problem_id"]
        # k = f"p{k}"
        v = r["min_cost"] or None
        d[k] = v
    return d

users = j["users"]
data = [results_to_dict(u["results"]) for u in users]
index = [u["team_name"] for u in users]

df = pd.DataFrame.from_records(data, index=index).astype("Int64")
# print(df)

# print(j["users"][-1]["results"])
# print(df.loc["raklo", 2])
# exit()

df = df.where(lambda v: v < 10 ** 6)
# print(df)
# exit()

baseline = {
    10: 2858,
    16: 13636,
    20: 28451,
}
cols = []
for size, full, lite in [
    (10, 26, 5),
    (10, 28, 10),
    (10, 29, 18),
    (16, 31, 24),
    (16, 32, 9),
    (16, 33, 15),
    (16, 34, 7),
    (16, 35, 25),
    (20, 27, 2),
    (20, 30, 11),
]:
    s = df.loc[:, full] - df.loc[:, lite] - baseline[size]
    s.name = f"{size} ({full}-{lite})"
    cols.append(s)

df2 = pd.concat(cols, axis=1)

df2.to_csv(f"out/{path.stem}.csv")

df2.index = df2.index.map(lambda name: name[:16])
with pd.option_context('display.max_rows', None, 'display.max_columns', None, 'display.width', None):
    print(df2)

