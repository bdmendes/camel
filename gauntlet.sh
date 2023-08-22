#!/usr/bin/env bash

MATCHER=fast-chess
REPO_URL=https://github.com/Disservin/fast-chess.git
REPO_TAG=v0.4
INSTALL_PATH=./$MATCHER
ENGINE_NAME=camel
BUILD_PATH=./target/release/camel
OUTPUT_FILE=results_tmp.txt
MESSAGE_FILE=message.txt
ROUNDS=200
TIME_CONTROL=10+0.1
THREADS=4
ELO_THRESHOLD=20

# Parse command line parameter
upstream=${1:-master}

# Clone fast-chess and compile it if not found
if ! command -v $INSTALL_PATH/$MATCHER &> /dev/null
then
    echo "$MATCHER not found at path, installing it from github..."
    if ! git clone $REPO_URL --branch $REPO_TAG --single-branch $INSTALL_PATH &> /dev/null
    then
        echo "Could not clone $MATCHER from git repository, aborting"
        exit 1
    fi
    
    if ! make -C $INSTALL_PATH
    then
        echo "Could not compile $MATCHER, aborting"
        exit 1
    fi
fi

# Save current branch name
current_branch=$(git branch --show-current)

# Compile current version and copy to fast-chess
echo "Compiling current version ($current_branch)"
if ! cargo build --release
then
    echo "Current engine version does not compile, aborting"
    exit 1
fi
cp -f $BUILD_PATH "$INSTALL_PATH/$ENGINE_NAME"

# If there are uncommited changes, stash them before switching to upstream
stashed=0
if ! git diff --quiet
then
    git stash
    stashed=1
fi

# Compile upstream version and copy to fast-chess
git checkout -d $upstream
echo "Compiling upstream version ($upstream)"
cargo build --release
cp -f $BUILD_PATH "$INSTALL_PATH/${ENGINE_NAME}_$upstream"

# Switch back to current branch
git checkout $current_branch

# If there were uncommited changes, unstash them
if [ $stashed == 1 ]
then
    git stash apply &> /dev/null
fi

# Enter fast-chess directory
cd $INSTALL_PATH

# If the files are the same, exit with success
if cmp -s "$ENGINE_NAME" "${ENGINE_NAME}_$upstream"
then
    echo -n "🆗 Engine executables do not differ: gauntlet skipped" | tee $MESSAGE_FILE
    echo ""
    exit
fi

# Run the gauntlet and store output in temp file
stdbuf -i0 -o0 -e0 ./${MATCHER} \
    -engine cmd=$ENGINE_NAME name=$ENGINE_NAME \
    -engine cmd=${ENGINE_NAME}_$upstream name=${ENGINE_NAME}_$upstream \
    -each tc=$TIME_CONTROL -rounds $ROUNDS -repeat -concurrency $THREADS | tee $OUTPUT_FILE

# Print last evaluation result
result=$(grep +/- $OUTPUT_FILE | tail -1)
result_array=($result)
elo_diff=${result_array[2]}
elo_diff_rounded=$(echo $elo_diff | awk '{printf("%d\n",$1 + 0.5)}')
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
echo -n $result | tee -a $MESSAGE_FILE
echo ""

# If the test failed, set exit code to 1
if [ $failed == 1 ]
then
    exit 1
fi