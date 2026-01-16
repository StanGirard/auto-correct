#!/bin/bash
set -e

NGRAMS_DIR="/ngrams"
FR_NGRAMS_URL="https://languagetool.org/download/ngram-data/ngrams-fr-20150913.zip"

# Check if French n-grams already exist
if [ ! -d "$NGRAMS_DIR/fr" ]; then
  echo "Downloading French n-grams..."
  mkdir -p "$NGRAMS_DIR"
  cd "$NGRAMS_DIR"

  # Download with progress
  curl -L -o ngrams-fr.zip "$FR_NGRAMS_URL"

  echo "Extracting French n-grams..."
  unzip -q ngrams-fr.zip
  rm ngrams-fr.zip

  echo "French n-grams installed successfully!"
else
  echo "French n-grams already installed."
fi

# Start LanguageTool (call the original start script)
cd /LanguageTool
exec bash start.sh "$@"
