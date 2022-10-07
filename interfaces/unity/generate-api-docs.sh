#!/bin/bash
# Copyright (c) Meta Platforms, Inc. and affiliates. 

exit_with_failure()
{
    echo "❌ $*" 1>&2 ; exit 1;
}

command -v doxygen &> /dev/null || exit_with_failure "Doxygen is not installed"

doxygen Doxygen.conf || exit_with_failure "Doxygen run failed"
