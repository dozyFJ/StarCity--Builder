#!/usr/bin/env bash
set -euo pipefail
if ! gh release view latest-apk > /dev/null 2>&1; then
  gh release create latest-apk --target "$GITHUB_SHA" --title "StarCity Builder release"
fi
gh release upload latest-apk build/starcity-builder-release.apk --clobber
gh release edit latest-apk --notes "Optimized release and OTA shell: $GITHUB_SHA. Updates: Pause → Обновить игру (OTA)."
if ! gh release view game-ota > /dev/null 2>&1; then
  gh release create game-ota --target "$GITHUB_SHA" --title "StarCity Builder native OTA"
fi
gh release upload game-ota build/game-ota/game-runtime.zip build/game-ota/game-ota.sig build/game-ota/game-ota.json --clobber
gh release edit game-ota --notes "Signed arm64 runtime: $GITHUB_SHA. Compatible shells update inside the game."
