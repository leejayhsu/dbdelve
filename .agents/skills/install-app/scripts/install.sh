#!/usr/bin/env bash
#
# Builds the release DBDelve.app and installs it into /Applications,
# replacing the copy already there.
#
# Usage: install.sh [--install-only]
#   --install-only  skip the build and install the existing target/DBDelve.app
#
# Exit codes:
#   0  installed
#   1  build or install failed
#   2  a build prerequisite is missing (Xcode, Metal toolchain)
#   3  the installed DBDelve is running; quit it, then rerun with --install-only
#   4  DBDelve is installed through Homebrew; replacing it would desync the cask
set -euo pipefail

ROOT="$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse --show-toplevel)"
SRC="$ROOT/target/DBDelve.app"
DEST="/Applications/DBDelve.app"
STAGE="/Applications/.DBDelve.app.installing"

fail() {
  echo "$2" >&2
  exit "$1"
}

[[ "$(uname)" == Darwin ]] || fail 1 "This installs a macOS app; this machine is $(uname)."

if [[ "${1:-}" != --install-only ]]; then
  xcrun -f metal >/dev/null 2>&1 ||
    fail 2 "No Metal compiler: install Xcode, then run: sudo xcode-select -s /Applications/Xcode.app/Contents/Developer"
  xcrun metal --version >/dev/null 2>&1 ||
    fail 2 "The Metal toolchain is missing: run xcodebuild -downloadComponent MetalToolchain (and sudo xcodebuild -runFirstLaunch first if that fails to load a plug-in)."

  (cd "$ROOT" && DBDELVE_CHANNEL=release dev/bundle.sh)
fi

[[ -d "$SRC" ]] || fail 1 "No build at $SRC; run without --install-only."

# A Homebrew cask owns its copy in /Applications; overwriting it leaves brew
# believing a version is installed that no longer is.
if [[ -d "$(brew --prefix 2>/dev/null)/Caskroom/dbdelve" ]]; then
  fail 4 "DBDelve is installed through Homebrew. Run: brew uninstall --cask dbdelve, then rerun with --install-only."
fi

# Replacing the bundle under a running process breaks it, so the user quits it
# first: DBDelve saves its buffers on quit, and only the user knows whether
# anything else is in flight.
if pgrep -f "$DEST/Contents/MacOS/" >/dev/null; then
  fail 3 "DBDelve is running from $DEST. Quit it, then rerun with --install-only."
fi

# Copied beside the old copy and verified before the old one goes, so a failed
# copy never leaves /Applications without a working DBDelve.
rm -rf "$STAGE"
ditto "$SRC" "$STAGE" || fail 1 "Could not write to /Applications."
codesign --verify --strict "$STAGE" || {
  rm -rf "$STAGE"
  fail 1 "The copied app's signature does not verify; the installed copy was left alone."
}
rm -rf "$DEST"
mv "$STAGE" "$DEST"

VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$DEST/Contents/Info.plist")"
echo "installed $DEST (v$VERSION)"
