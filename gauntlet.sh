#!/usr/bin/env bash

readonly RUNNER=fast-chess
readonly REPO_URL=https://github.com/Disservin/fast-chess.git
readonly REPO_TAG=v0.4
readonly INSTALL_PATH=./$RUNNER
readonly ENGINE_NAME=camel
readonly BUILD_PATH=./target/release/$ENGINE_NAME
readonly MESSAGE_FILE=message.txt
readonly ROUNDS=300
readonly TIME_CONTROL=10+0.1
readonly THREADS=4
readonly ELO_THRESHOLD=20
readonly UPSTREAM=${1:-master} # Default to master if no argument is given

if ! git diff --quiet
then
    echo "Commit or stash your changes first"
    exit 1
fi

if [ -z "$CURRENT_BRANCH" ]
then
    CURRENT_BRANCH=$(git branch --show-current)
    if [ -z "$CURRENT_BRANCH" ]
    then
        echo "Could not get current branch name (are you in a detached state?)"
        exit 1
    fi
else
    # In GH Actions, the current branch is already set
    echo "Current branch: $CURRENT_BRANCH"
fi

# Clone fast-chess and compile it if not found
if ! command -v $INSTALL_PATH/$RUNNER &> /dev/null
then
    echo "$RUNNER not found, cloning and compiling"
    rm -rf $INSTALL_PATH
    git clone $REPO_URL --branch $REPO_TAG --single-branch $INSTALL_PATH &> /dev/null || exit 1
    make -C $INSTALL_PATH || exit 1
fi

echo "Compiling current version ($CURRENT_BRANCH)"
cargo build --release || exit 1
cp $BUILD_PATH "$INSTALL_PATH/${ENGINE_NAME}-${CURRENT_BRANCH}" || exit 1

echo "Compiling upstream version ($UPSTREAM)"
git switch -d "$UPSTREAM" || exit 1
cargo build --release || exit 1
cp $BUILD_PATH "$INSTALL_PATH/${ENGINE_NAME}-$UPSTREAM" || exit 1

# Switch back to current branch
git switch "$CURRENT_BRANCH" || exit 1
cd $INSTALL_PATH || exit 1

if cmp -s "${ENGINE_NAME}-${CURRENT_BRANCH}" "${ENGINE_NAME}-$UPSTREAM"
then
    echo -n "🆗 Engine executables do not differ: gauntlet skipped" | tee $MESSAGE_FILE
    echo ""
    exit
fi

# Run the gauntlet and store output in temp file
OUTPUT_FILE=$(mktemp)
stdbuf -i0 -o0 -e0 ./${RUNNER} \
-engine cmd=${ENGINE_NAME}-"${CURRENT_BRANCH}" name=${ENGINE_NAME}-"${CURRENT_BRANCH}" \
-engine cmd=${ENGINE_NAME}-"$UPSTREAM" name=${ENGINE_NAME}-"$UPSTREAM" \
-each tc=$TIME_CONTROL -rounds $ROUNDS -repeat -concurrency $THREADS | tee "$OUTPUT_FILE"

# Print last evaluation result
result=$(grep +/- "$OUTPUT_FILE" | tail -1)
result_array=("$result")
elo_diff=${result_array[2]}
elo_diff_rounded=$(echo "$elo_diff" | awk '{printf("%d\n",$1 + 0.5)}')
failed=0
if [ $((elo_diff_rounded)) -lt -$ELO_THRESHOLD ]
then
    echo -n "❌ " | tee $MESSAGE_FILE
    failed=1
elif [ $((elo_diff_rounded)) -lt $ELO_THRESHOLD ]
then
    echo -n "🆗 " | tee $MESSAGE_FILE
    exit
else
    echo -n "✅ " | tee $MESSAGE_FILE
fi
echo -n "$result" | tee -a $MESSAGE_FILE
echo ""

if [ $failed == 1 ]
then
    exit 1
fi
