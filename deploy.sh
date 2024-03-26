#!/usr/bin/env bash

readonly RUNNER=lichess-bot
readonly REPO_URL=https://github.com/lichess-bot-devs/lichess-bot.git
readonly INSTALL_PATH=./$RUNNER
readonly CONFIG_PATH=./config.yml
readonly LOG_PATH=./lichess-bot.log
readonly ENGINE_NAME=camel

if ! git diff --quiet; then
    echo "Commit or stash your changes first"
    exit 1
fi

# Clone lichess-bot if directory does not exist
if [ ! -d "$INSTALL_PATH" ]; then
    echo "$RUNNER not found, cloning"
    git clone $REPO_URL --single-branch $INSTALL_PATH || exit 1
fi

# Compile the engine and copy it to the lichess-bot directory
cargo build --release || exit 1
cp ./target/release/$ENGINE_NAME ./$RUNNER/engines || exit 1

cd $INSTALL_PATH || exit 1

# Create virtual environment and install dependencies
python3 -m venv venv || exit 1
virtualenv venv -p python3 || exit 1
# shellcheck disable=SC1091
source ./venv/bin/activate || exit 1
python3 -m pip install -r requirements.txt || exit 1

# Error if the config file does not exist
if [ ! -f "$CONFIG_PATH" ]; then
    echo "Config file not found: $CONFIG_PATH"
    echo "Create a config file and try again"
    exit 1
fi

# Remove old log file
rm -f $LOG_PATH

# Create a new log file
touch $LOG_PATH

# Run the bot
python3 lichess-bot.py -v --logfile $LOG_PATH --config $CONFIG_PATH
