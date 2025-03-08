#! /usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

deployed_app_name="tarot"

source deployer.env

# Deploy assets with hashed filenames
s3_bucket=$(aws cloudformation list-exports --query "Exports[?Name=='${deployed_app_name}-WebAssetsBucket'].Value" --output text)

hash_filename() {
    filepath=$1
    the_hash=$(openssl dgst -md5 "$filepath" | awk '{ print $NF }' | cut -c 1-8)
    hashed_filename=$(basename "$filepath" | sed -E "s/^(.*)+\\.([^.]{1,5})\$/\\1.${the_hash}.\\2/")
    echo "${hashed_filename}"
}

SCRIPT_JS=$(hash_filename "assets/script.js")
STYLES_CSS=$(hash_filename "assets/styles.css")

aws s3 cp --cache-control "public,max-age=31536000,immutable" "assets/script.js" "s3://${s3_bucket}/assets/${SCRIPT_JS}"
aws s3 cp --cache-control "public,max-age=31536000,immutable" "assets/styles.css" "s3://${s3_bucket}/assets/${STYLES_CSS}"
aws s3 sync --cache-control "public,max-age=31536000,immutable" "assets/" "s3://${s3_bucket}/assets/" --exclude "script.js" --exclude "styles.css"


# Build the Lambda function
start_ts=$(date +%s)
echo "Building the Lambda function deployment package..."
(cd lambda && cargo lambda build --arm64 --release --output-format zip)
end_ts=$(date +%s)
elapsed=$((end_ts - start_ts))
echo "... built in ${elapsed}s."


# Deploy the Lambda function
function_name="${deployed_app_name}"
environment_variables="Variables={SCRIPT_JS=${SCRIPT_JS},STYLES_CSS=${STYLES_CSS}}"
zipfile="fileb://lambda/target/lambda/tarot-lambda/bootstrap.zip"

if ! aws lambda get-function --function-name "${function_name}" >/dev/null 2>&1; then
    echo "Lambda function ${function_name} does not exist.  Creating it..."
    start_ts=$(date +%s)

    lambda_role=$(aws cloudformation list-exports --query "Exports[?Name=='${deployed_app_name}-LambdaExecutionRoleArn'].Value" --output text)

    aws lambda create-function \
        --no-cli-pager \
        --function-name "${function_name}" \
        --handler 'bootstrap' \
        --runtime 'provided.al2023' \
        --architecture 'arm64' \
        --memory-size 128 \
        --zip-file "${zipfile}" \
        --role "${lambda_role}" \
        --environment "${environment_variables}" \
        --logging-config LogFormat=JSON
else
    echo "Updating Lambda function ${function_name}..."
    start_ts=$(date +%s)

    aws lambda update-function-code \
        --no-cli-pager \
        --function-name "${function_name}" \
        --zip-file "${zipfile}"

    aws lambda wait function-updated \
        --function-name "${function_name}"

    echo "  ...updating its configuration..."
    aws lambda update-function-configuration \
        --no-cli-pager \
        --function-name "${function_name}" \
        --environment "${environment_variables}"
fi

aws lambda wait function-updated \
    --function-name "${function_name}"

end_ts=$(date +%s)
elapsed=$((end_ts - start_ts))
echo "...deployed in ${elapsed}s."
