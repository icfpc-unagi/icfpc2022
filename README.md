# ICFPC2022
このファイルは提出時には USAGE.md にリネームします。

# 問題の提出方法

レポジトリ内に isl ファイルを用意し、その後に以下のコマンドを実行してください。

```
# (Optional) Dockerイメージを作り直します
make docker/tools
# Docker内で submit コマンドを実行します
docker run --rm -v $(pwd):/work -w /work icfpc-unagi/tools \
    ./bin/submit --logtostderr --problem=問題番号 --file=ISLファイル名
```
# 提出データの回収方法

```
# (Optional) Dockerイメージを作り直します
make docker/tools
# Docker内で collect_all コマンドを実行します
docker run --rm -v $(pwd):/work -w /work icfpc-unagi/tools \
    ./bin/collect_all --logtostderr
```

# 問題の回収方法

```
# (Optional) Dockerイメージを作り直します
make docker/tools
# Docker内で problems コマンドを実行します
docker run --rm -v $(pwd):/work -w /work icfpc-unagi/tools \
    ./bin/problems --logtostderr
```

# サーバの更新

```
cd go && make push
```

# ランナーの更新

```
make push/runner
```
