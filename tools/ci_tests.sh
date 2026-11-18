#!/usr/bin/env bash
set -euo pipefail
rustc --edition=2024 --test src/game.rs -o /tmp/starcity-builder-game-tests
cargo test --lib --target x86_64-unknown-linux-gnu
/tmp/starcity-builder-game-tests
python3 tools/test_ota_producer.py
mkdir -p /tmp/button-ota-tests
curl -fsSL --retry 3 https://repo.maven.apache.org/maven2/org/json/json/20240303/json-20240303.jar -o /tmp/button-ota-tests/json.jar
echo "3cf6cd6892e32e2b4c1c39e0f52f5248a2f5b37646fdfbb79a66b46b618414ed  /tmp/button-ota-tests/json.jar" | sha256sum -c -
javac --release 17 -cp /tmp/button-ota-tests/json.jar -d /tmp/button-ota-tests/classes android-shared/src/main/java/com/dozyfj/starcity/ota/BundleVerifier.java tools/OtaVerifierTest.java
java -ea -cp /tmp/button-ota-tests/classes:/tmp/button-ota-tests/json.jar OtaVerifierTest
