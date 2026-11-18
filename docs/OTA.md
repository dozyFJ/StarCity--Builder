# Native OTA and release builds

Adapted from dozyFJ/WolfCraft `codex/ota-launchers`, commit `1a075034764a8f4bbd173ce265a5a1e184389cd1`.
The stable Java APK shell loads the built-in or verified downloaded `libstarcity_builder.so`. Rust gameplay, UI, models and fonts update together.

## Flow

Install the release APK once. Open Pause → Обновить игру (OTA), check, download, and restart. Public GitHub updates require no token.
The game saves before opening the updater. Saved progress is separate from the OTA directory.
The updater can restore the previous core. If a new core dies before its first rendered GPU frame, the next startup rolls back.
OTA downloads a compressed full native core, not a binary delta. Android shell, permissions and save-format changes require a new APK.

## Trust

The manifest is RSA/SHA-256 signed by the same permanent key as the APK. The loader checks against the installed APK certificate.
It checks game identity, ARM64 ABI, shell API/fingerprint, save format, increasing build sequence, ZIP and per-file SHA-256 hashes and sizes.
Traversal, duplicates, extra entries, oversized archives and invalid ELF headers are rejected. Libraries are staged read-only and activated atomically; loaded code is never overwritten.
Only trusted GitHub hosts are allowed over HTTPS.

## Signing

Keep the private key out of this public repository and maintain a separate backup. Configure Actions Secrets:

- STAR_CITY_KEYSTORE_BASE64
- STAR_CITY_KEYSTORE_PASSWORD

The alias `androiddebugkey` is retained for tooling compatibility. The actual key is a private RSA release key, not a public/default Android debug key.
Without secrets, CI tests and uploads `starcity-builder-validation` (unsigned shell plus stripped native libraries); publication is skipped.
Initial manual signing can use the same permanent key, followed by tools/build_ota.py and tools/check_ota.sh.

## Validation

CI runs 14 game tests, producer tests and adversarial JVM verification tests, then compiles the optimized ARM64 release.
Checks cover ABI exports, stripped debug data, 16 KB page alignment, required shell classes, APK signature/alignment, and signature/extraction of the actual OTA archive.
The game-ota release contains game-ota.json, game-ota.sig, game-runtime.zip. The signed manifest is published last. Clients reject partial/inconsistent releases safely and can retry.
First installation, native restart, activation and rollback still require a real ARM64 Android phone test.
The permanent release signature differs from earlier random CI debug keys; preservation across that one-time signature change is not automatic.
