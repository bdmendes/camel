#!/usr/bin/env bash

readonly RUNNER=fast-chess
readonly REPO_URL=https://github.com/Disservin/fast-chess.git
readonly REPO_TAG=v0.7.0-alpha
readonly INSTALL_PATH=./$RUNNER
readonly ENGINE_NAME=camel
readonly BOOK_PATH=./assets/books
readonly BOOK_NAME=popularpos_lichess_v3.epd
readonly BOOK_FORMAT=epd
readonly BUILD_PATH=./target/release/$ENGINE_NAME
readonly MESSAGE_FILE=message.txt
readonly ELO_THRESHOLD=40

# Arguments
readonly UPSTREAM=${1:-"master"}
readonly CONCURRENCY_GAMES=${2:-"4"}
readonly ENGINE_HASH=${4:-"64"}
readonly ROUNDS=${5:-"500"}
readonly TIME_CONTROL=${6:-"5+0.2"}

function run_gauntlet {
    # (rounds, time_control)
    local rounds=$1
    local time_control=$2

    echo ""
    echo "Running gauntlet with $rounds rounds and $time_control time control against $UPSTREAM."
    echo "Settings: concurrency=$CONCURRENCY_GAMES; hash=$ENGINE_HASH."
    echo ""

    # Run the gauntlet and store output in temp file
    OUTPUT_FILE=$(mktemp)
    stdbuf -i0 -o0 -e0 ./${RUNNER} \
        -engine cmd="${CURRENT_BRANCH_BIN_NAME}" name="${CURRENT_BRANCH_BIN_NAME}" option.Hash="${ENGINE_HASH}" \
        -engine cmd="${UPSTREAM_BIN_NAME}" name="${UPSTREAM_BIN_NAME}" option.Hash="${ENGINE_HASH}" \
        -each tc="${time_control}" -rounds "${rounds}" -repeat -concurrency "${CONCURRENCY_GAMES}" -openings \
        file=${BOOK_NAME} format=${BOOK_FORMAT} order=random -draw movecount=8 score=8 movenumber=30 | tee "$OUTPUT_FILE"

    # Error if the elo difference line is not found
    result=$(grep +/- "$OUTPUT_FILE" | tail -1)
    if [ -z "$result" ]; then
        echo "Could not find result line in output"
        exit 1
    fi

    # Parse elo difference from output
    elo_diff=$(echo "$result" | grep -o -E '[+-]?[0-9]+([.][0-9]+)?' | head -1 | awk '{printf("%d\n",$1 + 0.5)}')

    # Print result
    failed=0
    echo -n "Against ${UPSTREAM} [hash=$ENGINE_HASH]: " | tee -a $MESSAGE_FILE
    if [ $((elo_diff)) -lt -$ELO_THRESHOLD ]; then
        echo -n "❌ " | tee -a $MESSAGE_FILE
        failed=1
    elif [ $((elo_diff)) -lt $ELO_THRESHOLD ]; then
        echo -n "🆗 " | tee -a $MESSAGE_FILE
    else
        echo -n "✅ " | tee -a $MESSAGE_FILE
    fi
    echo "$result" | tee -a $MESSAGE_FILE
    echo ""

    # Exit with error code if the new version is worse
    if [ $failed == 1 ]; then
        exit 1
    fi
}

if ! git diff --quiet; then
    echo "Commit or stash your changes first"
    exit 1
fi

if [ -z "$CURRENT_BRANCH" ]; then
    CURRENT_BRANCH=$(git branch --show-current)
    if [ -z "$CURRENT_BRANCH" ]; then
        CURRENT_BRANCH=$(git describe --tags --abbrev=0)
        if [ -z "$CURRENT_BRANCH" ]; then
            echo "Could not determine current branch"
            exit 1
        fi
    fi
else
    # In GH Actions, the current branch is already set
    echo "Current branch: $CURRENT_BRANCH"
fi

# Clone fast-chess and compile it if not found
if ! command -v $INSTALL_PATH/$RUNNER &>/dev/null; then
    echo "$RUNNER not found, cloning and compiling"
    rm -rf $INSTALL_PATH
    git clone $REPO_URL --branch $REPO_TAG --single-branch $INSTALL_PATH &>/dev/null || exit 1
    make -C $INSTALL_PATH || exit 1
fi

# Copy opening book if not found
if [ ! -f $INSTALL_PATH/${BOOK_NAME} ]; then
    echo "Opening book not found, copying"
    cp "${BOOK_PATH}/${BOOK_NAME}" "${INSTALL_PATH}/${BOOK_NAME}" || exit 1
fi

# Delete old binaries
rm $INSTALL_PATH/$ENGINE_NAME-*

# Use binary names based on branch name, with slashes replaced by dashes
CURRENT_BRANCH_BIN_NAME="${ENGINE_NAME}-${CURRENT_BRANCH//\//-}"
UPSTREAM_BIN_NAME="${ENGINE_NAME}-${UPSTREAM//\//-}"

echo "Compiling current version ($CURRENT_BRANCH)"
cargo build --release || exit 1
cp $BUILD_PATH "$INSTALL_PATH/${CURRENT_BRANCH_BIN_NAME}" || exit 1

echo "Compiling upstream version ($UPSTREAM)"
git switch -d "$UPSTREAM" || exit 1
cargo build --release || exit 1
cp $BUILD_PATH "$INSTALL_PATH/${UPSTREAM_BIN_NAME}" || exit 1

git switch -f "$CURRENT_BRANCH" || exit 1

cd $INSTALL_PATH || exit 1

# Truncate message file
echo -n "" >$MESSAGE_FILE

run_gauntlet "$ROUNDS" "$TIME_CONTROL"
