import json
import glob
import time
import subprocess
import fire



def find_best_submissions():
    best_submissions = {}
    for path in glob.glob("submissions/*.json"):
        s = json.load(open(path))
        if s["status"] != "SUCCEEDED":
            continue

        pid = s["problem_id"]
        if (s["problem_id"] not in best_submissions) or s["cost"] < best_submissions[pid]["cost"]:
            best_submissions[pid] = s

    return best_submissions


def find_local_best_solution(problem_id):
    paths = glob.glob(f"out/opt_{problem_id}_*")
    if len(paths) == 0:
        return None

    get_score = lambda path: int(path.split('_')[2])
    paths.sort(key=get_score)
    path = paths[0]
    return path, get_score(path)


def main(force_submit_best=False):
    best_submissions = find_best_submissions()

    for problem_id, best_submission in sorted(best_submissions.items()):
        best_submission_score = int(best_submission["cost"])
        best_local_solution = find_local_best_solution(problem_id)

        if best_local_solution is None:
            print(f"Problem {problem_id:3d}: Solution not found")
            continue
        
        new_path, new_score = best_local_solution
        if force_submit_best:
            assert new_score <= best_submission_score
        else:
            if new_score >= best_submission_score:
                print(f"Problem {problem_id:3d}: Local solution not better ({best_submission_score} <= {new_score})")
                continue

        print(f"Problem {problem_id:3d}: Submitting! ({best_submission_score} > {new_score})")

        cmd = f'curl -X POST --data-urlencode isl@{new_path} -d problem_id={problem_id} "https://icfpc.sx9.jp/scvzcaae/submit"'
        print(cmd)
        subprocess.run(cmd, shell=True)
        time.sleep(3)



if __name__ == "__main__":
    import fire  # pip install fire
    fire.Fire(main)