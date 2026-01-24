#!/bin/bash
# Upload N-gram data to Cloudflare R2 (maintainers only)
#
# Prerequisites:
#   1. Install rclone: https://rclone.org/install/
#   2. Configure R2 remote:
#      rclone config
#      - Name: r2
#      - Type: s3
#      - Provider: Cloudflare
#      - Access Key: <from R2 dashboard>
#      - Secret Key: <from R2 dashboard>
#      - Endpoint: https://<account-id>.r2.cloudflarestorage.com
#
# Usage:
#   ./scripts/upload_ngrams_r2.sh en
#   ./scripts/upload_ngrams_r2.sh fr
#   ./scripts/upload_ngrams_r2.sh all

set -e

# R2 bucket name
R2_BUCKET="${GRAMMAR_RS_R2_BUCKET:-autocorrect-quivr}"
R2_REMOTE="${GRAMMAR_RS_R2_REMOTE:-r2}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="${SCRIPT_DIR}/../data/ngrams"

upload_file() {
    local lang=$1
    local filename="${lang}_ngrams.bin"
    local source="${DATA_DIR}/${filename}"

    if [ ! -f "$source" ]; then
        echo "Error: ${source} not found"
        echo "Build it first with: cargo run --bin sync-lt -- --extract-ngrams --language ${lang}"
        exit 1
    fi

    local size=$(stat -f%z "$source" 2>/dev/null || stat -c%s "$source" 2>/dev/null)
    echo "Uploading ${filename} ($(numfmt --to=iec-i --suffix=B $size 2>/dev/null || echo "$size bytes"))..."

    # Upload with progress
    rclone copy \
        --progress \
        --s3-chunk-size 64M \
        --transfers 4 \
        "$source" \
        "${R2_REMOTE}:${R2_BUCKET}/ngrams/"

    echo "  Uploaded to: ${R2_REMOTE}:${R2_BUCKET}/ngrams/${filename}"

    # Generate and upload checksum
    local checksum_file="${source}.sha256"
    echo "Generating SHA256 checksum..."
    sha256sum "$source" | awk '{print $1}' > "$checksum_file"
    echo "  Checksum: $(cat "$checksum_file")"

    rclone copy \
        "$checksum_file" \
        "${R2_REMOTE}:${R2_BUCKET}/ngrams/"

    echo "  Uploaded checksum to: ${R2_REMOTE}:${R2_BUCKET}/ngrams/${filename}.sha256"
}

verify_rclone() {
    if ! command -v rclone &> /dev/null; then
        echo "Error: rclone not installed"
        echo "Install: https://rclone.org/install/"
        exit 1
    fi

    if ! rclone listremotes | grep -q "^${R2_REMOTE}:"; then
        echo "Error: rclone remote '${R2_REMOTE}' not configured"
        echo "Run 'rclone config' to set up Cloudflare R2"
        exit 1
    fi
}

case "${1:-}" in
    en|EN)
        verify_rclone
        upload_file "en"
        ;;
    fr|FR)
        verify_rclone
        upload_file "fr"
        ;;
    all|ALL)
        verify_rclone
        upload_file "en"
        upload_file "fr"
        ;;
    *)
        echo "Grammar-RS N-gram Data Uploader (R2)"
        echo ""
        echo "Usage: $0 <language>"
        echo ""
        echo "Languages:"
        echo "  en   - Upload English N-grams"
        echo "  fr   - Upload French N-grams"
        echo "  all  - Upload both"
        echo ""
        echo "Prerequisites:"
        echo "  1. rclone installed and configured with R2 credentials"
        echo "  2. N-gram .bin files built in data/ngrams/"
        echo ""
        echo "Environment variables:"
        echo "  GRAMMAR_RS_R2_BUCKET - R2 bucket name (default: grammar-rs-ngrams)"
        echo "  GRAMMAR_RS_R2_REMOTE - rclone remote name (default: r2)"
        exit 1
        ;;
esac

echo ""
echo "Upload complete!"
echo ""
echo "To make files public, configure bucket settings in Cloudflare dashboard:"
echo "  1. Go to R2 > ${R2_BUCKET} > Settings"
echo "  2. Enable 'Public access'"
echo "  3. Note the public URL: https://pub-<id>.r2.dev"
echo "  4. Update GRAMMAR_RS_R2_URL in download script"
