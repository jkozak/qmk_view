#!/usr/bin/env bash
set -euo pipefail

# Create a self-extracting archive with all dependencies
OUTPUT="qmkview-standalone"
TMPDIR=$(mktemp -d)

echo "Creating standalone bundle..."

# Copy the nix result into temp directory
cp -rL result "$TMPDIR/app"

# Create the self-extracting script
cat > "$OUTPUT" << 'EXTRACT_EOF'
#!/usr/bin/env bash
set -euo pipefail

# Self-extracting archive
SKIP_LINES=__SKIP_LINES__
TMPDIR="${TMPDIR:-/tmp}"
EXTRACT_DIR="$TMPDIR/qmkview-$$"

# Extract the payload
tail -n +$SKIP_LINES "$0" | tar xzf - -C "$TMPDIR"
trap "rm -rf '$EXTRACT_DIR'" EXIT

# Run the application
exec "$EXTRACT_DIR/app/bin/qmkview" "$@"
EXTRACT_EOF

# Calculate the number of lines in the header
SKIP_LINES=$(( $(wc -l < "$OUTPUT") + 1 ))
sed -i "s/__SKIP_LINES__/$SKIP_LINES/" "$OUTPUT"

# Append the tarball
tar czf - -C "$TMPDIR" app >> "$OUTPUT"

# Make it executable
chmod +x "$OUTPUT"

# Cleanup
rm -rf "$TMPDIR"

echo "Created: $OUTPUT ($(du -h "$OUTPUT" | cut -f1))"
