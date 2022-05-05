#!/bin/sh

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

command -v cargo-license &> /dev/null || exit_with_failure "cargo-license is not installed"
command -v cargo-workspaces &> /dev/null || exit_with_failure "cargo-workspaces is not installed"

[[ -z $(git status -s) ]] || exit_with_failure "git working directory not clean"

LICENSE_FILE=licenses/3RD-PARTY-LICENSES.md

echo "➡️ Collecting licenses"

# Creates a list of local crates, separated by '|'
#
# Example output: api|clip-players|datamodel|dsp|haptic-renderer
#
# - tr changes the crate separator from newline to |
# - sed removes the last newline
LOCAL_CRATES=$(cargo workspaces list | tr '\n' '|' | sed 's/.$//')

# Creates a list of all Rust dependencies
#
# Example output:
#    addr2line: Apache-2.0 OR MIT
#    adler: 0BSD OR Apache-2.0 OR MIT
#    [..]
#
# - awk picks out the crate name and license columns, ignoring other columns such
#   as version name and authors
# - egrep removes all local crates
# - uniq removes duplicates - for example the "syn" crate appears twice in the output,
#   since it gets included twice with different versions
# - sed removes the table header line
ALL_RUST_DEPS=$(cargo-license --do-not-bundle --tsv \
    | awk -F"\t" '{print $1 ": " $5}' \
    | egrep -v $LOCAL_CRATES \
    | uniq \
    | sed 1d)

echo "➡️ Writing licenses to $LICENSE_FILE"

cat <<EOF > $LICENSE_FILE
# Rust Core

$ALL_RUST_DEPS

# Android

$(<licenses/licenses-android.template)

# iOS

$(<licenses/licenses-ios.template)

# Licenses

$(<licenses/licenses-types.template)
EOF

echo "➡️ Committing license changes"
git add $LICENSE_FILE || exit_with_failure "Failed to stage changes"
git commit -m "Update license file" ||  exit_with_failure "Failed to commit changes"
