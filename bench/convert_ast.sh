BENCH_DIR=$PWD

cd ../nl2ast && sbt "run file://$BENCH_DIR/models/$1/model.nlogox $BENCH_DIR/models/$1/ast.json"
