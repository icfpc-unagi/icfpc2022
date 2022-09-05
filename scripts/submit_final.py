import json
import glob
import time
import subprocess
import os


def find_best_submissions():
    best_submissions = {}
    for path in glob.glob("/home/takiba/Dropbox/ICFPC2022/runs/*.json"):
        s = json.load(open(path))

        pid = s["problem_id"]
        if (s["problem_id"] not in best_submissions) or s["cost"] < best_submissions[pid][0]["cost"]:
            best_submissions[pid] = (s, path)

    return best_submissions


def main(force_submit_best=False, dryrun=False):
    best_submissions = find_best_submissions()

    for problem_id, (submission, path) in sorted(best_submissions.items()):
        print(f"{problem_id:3d}  {submission['cost']:8d}   {path}")  # {submission['id']:5d}

    for problem_id, (submission, json_path) in sorted(best_submissions.items()):
        isl_path = os.path.splitext(json_path)[0] + ".isl"

        cmd = f'curl -X POST --data-urlencode isl@{isl_path} -d problem_id={problem_id} "https://icfpc.sx9.jp/scvzcaae/submit"'
        print(cmd)
        if not dryrun:
            subprocess.run(cmd, shell=True)
            time.sleep(3)


if __name__ == "__main__":
    import fire  # pip install fire
    fire.Fire(main)