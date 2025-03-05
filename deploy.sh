#! /usr/bin/env bash

set -o errexit
set -o pipefail
set -o nounset

# Deploy assets with hashed filenames
s3_bucket="tarot-rs-web-assets"

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


# Deploy the Lambda function
function_name="tarot-rs"

start_ts=$(date +%s)
echo "Building the ${function_name} Lambda function deployment package..."
(cd lambda && cargo lambda build --arm64 --release --output-format zip)
end_ts=$(date +%s)
elapsed=$((end_ts - start_ts))
echo "... built in ${elapsed}s."


echo "Updating Lambda function ${function_name}..."
start_ts=$(date +%s)

aws lambda update-function-code \
    --no-cli-pager \
    --function-name "${function_name}" \
    --zip-file "fileb://lambda/target/lambda/tarot-lambda/bootstrap.zip"

aws lambda wait function-updated \
    --function-name "${function_name}"

echo "  ...updating its configuration..."
aws lambda update-function-configuration \
    --no-cli-pager \
    --function-name "${function_name}" \
    --environment "Variables={SCRIPT_JS=${SCRIPT_JS},STYLES_CSS=${STYLES_CSS}}"

aws lambda wait function-updated \
    --function-name "${function_name}"

end_ts=$(date +%s)
elapsed=$((end_ts - start_ts))
echo "...deployed in ${elapsed}s."


function_url=$(aws lambda get-function-url-config --function-name "${function_name}" --output text --query FunctionUrl)
echo "Function is available at: ${function_url}"