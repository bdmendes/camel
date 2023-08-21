#!/usr/bin/env bash

MATCHER=fast-chess
REPO_URL=https://github.com/Disservin/fast-chess.git
REPO_TAG=v0.4
INSTALL_PATH=./$MATCHER
ENGINE_NAME=camel
BUILD_PATH=./target/release/camel
UPSTREAM=master
NUM_THREADS=$(nproc --all)
OUTPUT_FILE=results_tmp.txt
MESSAGE_FILE=message.txt
ROUNDS=200
TIME_CONTROL=10+0.1
THREADS=4
ELO_THRESHOLD=20

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
git switch $UPSTREAM
echo "Compiling upstream version ($UPSTREAM)"
cargo build --release
cp -f $BUILD_PATH "$INSTALL_PATH/${ENGINE_NAME}_$UPSTREAM"

# Switch back to current branch
git checkout $current_branch

# If there were uncommited changes, unstash them
if [ $stashed == 1 ]
then
    git stash apply &> /dev/null
fi

# Run the gauntlet and store output in temp file
path=$(pwd)
cd $INSTALL_PATH
stdbuf -i0 -o0 -e0 ./${MATCHER} \
    -engine cmd=$ENGINE_NAME name=$ENGINE_NAME \
    -engine cmd=${ENGINE_NAME}_$UPSTREAM name=${ENGINE_NAME}_$UPSTREAM \
    -each tc=$TIME_CONTROL -rounds $ROUNDS -repeat -concurrency $THREADS | tee $OUTPUT_FILE

# Print last evaluation result
result=$(grep +/- $OUTPUT_FILE | tail -1)
echo -n $result | tee $MESSAGE_FILE

result_array=($result)
elo_diff=${result_array[2]}
elo_diff_rounded=$(echo $elo_diff | awk '{printf("%d\n",$1 + 0.5)}')

if [ $((elo_diff_rounded)) -lt -$ELO_THRESHOLD ]
then
    echo -n " ❌" | tee -a $MESSAGE_FILE
    exit
fi

if [ $((elo_diff_rounded)) -lt $ELO_THRESHOLD ]
then
    echo -n " 🆗" | tee -a $MESSAGE_FILE
    exit
fi

echo -n " ✅" | tee -a $MESSAGE_FILE