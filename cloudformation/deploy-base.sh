#! /usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

app_name="${1:-tarot}"

aws cloudformation deploy \
    --no-cli-pager \
    --stack-name "${app_name}-base" \
    --template-file "base.yml" \
    --capabilities CAPABILITY_NAMED_IAM \
    --parameter-overrides "AppName=${app_name}"
