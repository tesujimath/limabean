#!/usr/bin/env bash
#
# encapsulate getting the version, since its location is subject to change
grep '^version' rust/limabean/Cargo.toml | head -1 | cut -d'"' -f2
