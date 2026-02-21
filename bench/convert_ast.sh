SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
PROJECT_ROOT=$(dirname $SCRIPT_DIR)

if [ -z "$1" ]; then
    echo "Usage: $0 <model_name>"
    exit 1
fi
MODEL_NAME=$1

cd $PROJECT_ROOT/nl2ast && sbt "run file://$SCRIPT_DIR/models/$1/model.nlogox $SCRIPT_DIR/models/$1/ast.json"
