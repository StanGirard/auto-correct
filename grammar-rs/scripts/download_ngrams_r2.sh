#!/bin/bash
# Download pre-built N-gram data from Cloudflare R2
#
# Usage:
#   ./scripts/download_ngrams_r2.sh en    # Download English (~23GB)
#   ./scripts/download_ngrams_r2.sh fr    # Download French (~6GB)
#   ./scripts/download_ngrams_r2.sh all   # Download both

set -e

# R2 public bucket URL
R2_BASE_URL="${GRAMMAR_RS_R2_URL:-https://pub-8068a615549c43e1893eb3f9a35a0e17.r2.dev/ngrams}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/../data/ngrams"

download_file() {
    local lang=$1
    local filename="${lang}_ngrams.bin"
    local url="${R2_BASE_URL}/${filename}"
    local output="${DATA_DIR}/${filename}"

    echo "Downloading ${filename}..."

    # Create data directory
    mkdir -p "${DATA_DIR}"

    # Check if file exists and get size
    if [ -f "$output" ]; then
        local local_size=$(stat -f%z "$output" 2>/dev/null || stat -c%s "$output" 2>/dev/null)
        echo "  Existing file: ${local_size} bytes"

        # Get remote size via HEAD request
        local remote_size=$(curl -sI "$url" | grep -i content-length | awk '{print $2}' | tr -d '\r')

        if [ "$local_size" = "$remote_size" ]; then
            echo "  File is up to date, skipping download"
            return 0
        fi
        echo "  Size mismatch (remote: ${remote_size}), re-downloading..."
    fi

    # Download with progress
    if command -v curl &> /dev/null; then
        curl -L --progress-bar -o "$output" "$url"
    elif command -v wget &> /dev/null; then
        wget --show-progress -O "$output" "$url"
    else
        echo "Error: curl or wget required"
        exit 1
    fi

    # Verify download
    if [ -f "$output" ]; then
        local size=$(stat -f%z "$output" 2>/dev/null || stat -c%s "$output" 2>/dev/null)
        echo "  Downloaded: $(numfmt --to=iec-i --suffix=B $size 2>/dev/null || echo "$size bytes")"
    else
        echo "Error: Download failed"
        exit 1
    fi
}

case "${1:-}" in
    en|EN)
        download_file "en"
        ;;
    fr|FR)
        download_file "fr"
        ;;
    all|ALL)
        download_file "en"
        download_file "fr"
        ;;
    *)
        echo "Grammar-RS N-gram Data Downloader (R2)"
        echo ""
        echo "Usage: $0 <language>"
        echo ""
        echo "Languages:"
        echo "  en   - English (~23GB)"
        echo "  fr   - French (~6GB)"
        echo "  all  - Both languages (~29GB)"
        echo ""
        echo "Data will be downloaded to: ${DATA_DIR}"
        echo ""
        echo "Environment variables:"
        echo "  GRAMMAR_RS_R2_URL - Custom R2 bucket URL"
        exit 1
        ;;
esac

echo ""
echo "Download complete! N-gram data ready for use."
